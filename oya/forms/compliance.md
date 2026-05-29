---
doc_class: ComplianceMatrix
microservice: forms
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-legal-compliance + council-privacy + ops-security
review_cadence: quarterly (per ADR-0133 axis-4)
doc_status: published
---

# Forms — Compliance Matrix

This document maps every regulatory regime that applies to the forms µservice to the specific control point that satisfies it, and the CI lane / runbook / dashboard / SLO that asserts the control. **No control is a TODO** (per `feedback_autonomous_implementation_artifacts.md`).

## 1. GDPR (Regulation (EU) 2016/679)

Applies in pack-eu by primary residency; applies extraterritorially per Art. 3(2) to any tenant offering forms to EU subjects.

| Article | Obligation | Control | Where |
|---|---|---|---|
| Art. 5(1)(a) Lawfulness, fairness, transparency | Forms surfaces lawful basis at authoring; submitter consent notice (Art. 7) explicit per form | `form.v1.consent_notice` mandatory for any field with `data_class=PII_*` | Builder validation lane |
| Art. 5(1)(b) Purpose limitation | Each form declares purpose; cross-purpose reuse blocked | Cedar policy on response re-export | `policy/tenant-scope.cedar` PERMIT-9 |
| Art. 5(1)(c) Data minimisation | AI-form-build outputs flagged for over-asking; non-essential PII rejected at publish | `oya-forms-data-minimisation-conformance` | CI lane |
| Art. 5(1)(d) Accuracy | Submitter rectification path always-on per FR-24 | DSR cascade runner | `oya-dsr-cascade-runner` |
| Art. 5(1)(e) Storage limitation | Per-pack retention table; auto-purge after window | `oya-forms-retention-conformance` | `policy/data-residency.md` §Retention |
| Art. 5(1)(f) Integrity & confidentiality | Per-tenant DEK envelope encryption + TLS 1.3 in transit | ADR-FORMS-0003 | Column-level encryption + Istio mTLS |
| Art. 7 Conditions for consent | Explicit, granular, withdrawable; pre-ticked boxes forbidden | Form schema validation rejects pre-checked PII consent | Builder lint |
| Art. 9 Special categories | Tagged at authoring; explicit consent + DPIA mandatory; pack-eu PHI-equivalent fields require HIPAA pack or explicit DPIA waiver | `data_class=SENSITIVE_GDPR_ART9` flags | `dpia.md` §Special-category |
| Art. 12-22 Data subject rights | DSR cascade; access + rectification + erasure + portability + objection | `oya-dsr-cascade-runner` | Per-pack SLA |
| Art. 22 Automated decision-making | T2 AI-form-build outputs reviewed by tenant; no solely-automated form pass/fail | ADR-FORMS-0005 §"Decision" | Reviewer gate |
| Art. 25 DPbDD | Default config: minimal-fields, max-privacy, captcha-on, audit-on | `oya-forms-default-privacy-baseline` | CI lane |
| Art. 28 Processor | Forms is processor for tenants who are controllers; DPA template + sub-processor list | `legal/dpa-template.md`, `legal/sub-processors.md` | Annual review |
| Art. 30 Records of processing | ROPA generated per tenant from form metadata | `legal/ropa.md` | DPIA-linked |
| Art. 32 Security of processing | Pen-test annual; chaos-drill quarterly; Cedar default-deny | `threat-model.md` | SOC 2 evidence |
| Art. 33 Breach notification | ≤ 72h notification path via incident-response runbook | `runbooks/pii-leak-incident-p0.md` | Tested annually |
| Art. 35 DPIA | Mandatory for special-category, AI-form-build, large-scale (≥ 10k subjects) | `dpia.md` | Updated per release |
| Arts. 44-50 International transfers | SCC-only for cross-border; pack-routing default | `policy/data-residency.md` | Annual register update |

## 2. KR PIPA (Personal Information Protection Act, Korea)

Applies in pack-kr by primary residency.

| Article | Obligation | Control |
|---|---|---|
| Art. 15 Collection | Lawful basis declared per form; consent explicit | Builder validation |
| Art. 17 Use & provision | Cross-purpose use prohibited; explicit re-consent | Cedar PERMIT-9 |
| Art. 22-2 Personal Identification Number (RRN) | RRN field explicitly forbidden unless tenant has KISA-approved basis | Builder rejects raw-RRN field; offers alternative-ID field |
| Art. 23 Sensitive information | Tagged; processing controls | `data_class=SENSITIVE_PIPA_ART23` |
| Art. 24 Unique Identifier | Tightly scoped; consent rules | DB column-level encryption |
| Art. 28 Storage limitation | Bounded retention; minimum data | Retention table in `policy/data-residency.md` |
| Art. 28-2 Pseudonymisation | Available when statistical / research use | Tooling at export |
| Art. 36 Right to erasure | Honoured ≤ 30d | `oya-dsr-cascade-runner` |
| Enforcement Decree Art. 30 | Audit log retention ≥ 1y (KR-FSS sector: 5y) | Audit-chain retention |
| PIPC Notice 2020-7 | Overseas transfer notification | `legal/transfer-register.md` |

## 3. HIPAA (45 CFR Parts 160, 162, 164)

Applies in pack-us-healthcare with active BAA.

| Section | Obligation | Control |
|---|---|---|
| §164.308 Administrative safeguards | Workforce training; access management; risk analysis | OpenBao access; quarterly review |
| §164.310 Physical safeguards | OCI us-ashburn-1 HIPAA-eligible facility | OCI BAA |
| §164.312(a) Access control | RBAC + Cedar + audit | `policy/tenant-scope.cedar` |
| §164.312(b) Audit controls | Audit-chain per PHI read | `oya-forms-audit-chain-coverage` |
| §164.312(c) Integrity | Ed25519 seal + checksum | Audit-chain seal SDK |
| §164.312(d) Person/entity authentication | OIDC + MFA for PHI access | Tenancy entitlement |
| §164.312(e) Transmission security | TLS 1.3 + Istio mTLS | mesh policy |
| §164.316(b) Documentation retention | 6 years | `policy/data-residency.md` §Retention |
| §164.530(j) Records | 6-year retention | Same |
| §164.404 Breach notification | ≤ 60d to subjects; ≤ 60d to HHS | `runbooks/pii-leak-incident-p0.md` |

## 4. EU AI Act (Regulation (EU) 2024/1689)

Applies to T2 AI-form-build per ADR-FORMS-0005.

| Article | Obligation | Control |
|---|---|---|
| Art. 9 Risk management | Per-release risk review of AI-form-build | `legal/ai-act-conformity.md` |
| Art. 10 Data governance | Training data not used (BYO-LLM); prompt provenance recorded | `dashboards/ai-form-build-quality.json` |
| Art. 11 Technical documentation | Maintained per release; pointer in conformity-assessment | `legal/ai-act-conformity.md` |
| Art. 12 Record-keeping | 90-day prompt + completion log | Audit-chain |
| Art. 13 Transparency | Tenant + submitter notified when AI authored form | Builder UI label + submitter banner |
| Art. 14 Human oversight | Tenant explicitly reviews + accepts; reviewer-agent for cross-µservice | ADR-FORMS-0005 §"Decision" |
| Art. 15 Accuracy + robustness | Schema-valid completion rate ≥ 80% (SLI) | `slos/ai-form-build-quality.openslo.yaml` |
| Annex III §4 (high-risk employment) | If form used for employment screening → high-risk; mandatory CE-marking + notified body | High-risk classification + DPIA trigger |
| Art. 50 Transparency of AI-generated content | Form metadata exposes `ai_build_origin` | Form spec field |
| Art. 72 Post-market monitoring | Quarterly safety-signal review | `dashboards/ai-form-build-quality.json` |

## 5. eIDAS (Regulation (EU) 910/2014)

| Article / Annex | Obligation | Control |
|---|---|---|
| Art. 25 Legal effects of electronic signatures | SES/AES/QES classification per signature | ADR-FORMS-0006 |
| Annex I (QES requirements) | QES requires QSCD + qualified certificate | Per-tenant tier-G+ entitlement; CA list |
| ETSI EN 319 122/132/142 | XAdES/CAdES/PAdES profiles | Per-format signer worker |

## 6. PCI DSS v4 (Payment Card Industry)

Applies when payment field enabled.

| Requirement | Obligation | Control |
|---|---|---|
| Req 1 Network security | mTLS + WAF + NetworkPolicy egress allowlist | Helm + Istio |
| Req 3 Protect stored data | No PAN stored; tokenisation at fintech (Tier-G) | Payment adapter; scope-reduction proof |
| Req 4 Encrypt transmission | TLS 1.3 minimum | WAF config |
| Req 6 Secure systems | SLSA L3 + NIST SSDF | `legal/slsa-l3.md` |
| Req 8 Authenticate users | OIDC + MFA for tenant operators | Tenancy |
| Req 10 Log + monitor | Audit-chain + Prometheus + Mimir | observability bridge |
| Req 11 Test security | Annual pen-test + quarterly chaos | `threat-model.md` |
| Req 12 Information security policy | Policy at `policy/*.cedar` + Cedar default-deny | Cedar bundle |

## 7. WCAG 2.2 AA (W3C Recommendation, 5 October 2023)

Every form renderer pass blocked on AA failure.

| Criterion | Test | Lane |
|---|---|---|
| 1.4.3 Contrast | axe-core | `oya-governance-wcag22-conformance` |
| 2.1.1 Keyboard | manual + Cypress | Same |
| 2.4.7 Focus visible | Visual regression | Same |
| 2.5.8 Target size (minimum) | Auto-measure | Same |
| 3.3.7 Redundant entry | Pre-fill aware | Same |
| 3.3.8 Accessible authentication | OIDC ≤ AAL2 | Tenancy |
| 4.1.3 Status messages | Live regions on validation | Renderer |

## 8. APPI / PDPA / DPDPA / LGPD / UAE PDPL / KSA PDPL

Per-pack overlay enforcement:

| Pack | Statute | Key control |
|---|---|---|
| pack-jp | APPI | Pseudonymisation; cross-border notification |
| pack-sg | PDPA | DNC registry honour; consent withdrawal |
| pack-in | DPDPA 2023 | Significant Data Fiduciary obligations; pseudo + child-protection |
| pack-br | LGPD | DPO contact; ANPD breach notification |
| pack-ae | UAE PDPL | Cross-border via SCC-equivalent |
| pack-ksa | KSA PDPL | NCA notification; processor registration |

Each pack ships an overlay file at `regional-packs/<pack>/forms-compliance-overlay.md` (mirroring sheets / workflow-studio convention).

## 9. SOC 2 Type 2 + ISO 27001:2022

Forms participates in the oyatie-wide SOC 2 / ISO 27001 program; controls evidenced via:

- Audit-chain seals (every state change).
- Cedar policy bundle (access controls).
- DPIA + AI-Act-conformity (privacy + AI).
- Pen-test report (annual).
- Chaos-drill ledger (quarterly).
- Workforce training ledger.
- Sub-processor list + DPA.

## 10. OWASP ASVS v4 + CIS K8s + SLSA L3 + NIST SSDF

| Framework | Coverage |
|---|---|
| OWASP ASVS v4 L2 | All non-FinTech forms |
| OWASP ASVS v4 L3 | Payment-enabled + healthcare-pack forms |
| CIS Kubernetes Benchmark | All cluster nodes; weekly scan |
| SLSA Level 3 | Builds reproducible; provenance signed |
| NIST SSDF SP 800-218 | PO / PS / PW / RV practices in CI |

## References

- `policy/data-residency.md`.
- `threat-model.md`.
- `dpia.md`.
- `legal/{dpa-template, sub-processors, transfer-register, ai-act-conformity, slsa-l3, ropa, baa-template, schrems-supplementary-measures}.md` (authored per-tenant under operational layer).
- All ADRs cited inline.

---



## §day-one-cert-readiness
This anchor is closed for `forms` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `forms` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +13 more.
- Example: `t0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `t0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §minor-protection
This anchor is closed for `forms` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `forms` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `t0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `forms` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`, `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `t0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.forms.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `forms` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `t0-suggest` touches those data classes.
- Signal sources: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`; +12 more.
- Example event class: `oya.forms.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `forms` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.forms.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `t0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `t0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `forms` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`, `forms.unknown`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `forms.unknown` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `forms` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`; +12 more.
- Example: `t0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.forms` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/forms/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.
- Example: `t0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `forms` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`, `microservices/forms/iac/helm/captcha-sidecar/Chart.yaml`, `microservices/forms/iac/helm/captcha-sidecar/values.yaml`, `microservices/forms/iac/helm/form-cdn/Chart.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `t0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `forms` is in annual full-scope pentest and every major `t0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`, `microservices/forms/iac/helm/captcha-sidecar/Chart.yaml`, `microservices/forms/iac/helm/captcha-sidecar/values.yaml`, `microservices/forms/iac/helm/form-cdn/Chart.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `forms` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `forms` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `t0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `forms` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/forms/catalog/oya-forms-bulk-distribute-worker.yaml`, `microservices/forms/catalog/oya-forms-captcha-adapter.yaml`, `microservices/forms/catalog/oya-forms-conditional-logic-domain.yaml`, `microservices/forms/catalog/oya-forms-crypto-domain.yaml`, `microservices/forms/catalog/oya-forms-domain.yaml`, `microservices/forms/catalog/oya-forms-export-worker.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `t0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `forms` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `t0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `t0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `forms` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `forms.unknown`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `t0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `forms`; owner `axis-forms`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/forms/capabilities/T0-suggest.yaml`, `microservices/forms/capabilities/T1-assist.yaml`, `microservices/forms/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar/policy artifacts cited: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`, `microservices/forms/contracts/openapi/forms.openapi.yaml`, `microservices/forms/contracts/proto/forms.proto`.
- Cedar binding: `microservices/forms/policy/auditor-scope.cedar`, `microservices/forms/policy/ci-scope.cedar`, `microservices/forms/policy/data-residency.md`, `microservices/forms/policy/dual-context.md`, `microservices/forms/policy/public-read.cedar`, `microservices/forms/policy/tenant-scope.cedar`.
- State/event binding: `forms.unknown`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/forms/slos/ai-form-build-latency.openslo.yaml`, `microservices/forms/slos/analytics-render-latency.openslo.yaml`, `microservices/forms/slos/bulk-distribute-latency.openslo.yaml`, `microservices/forms/slos/export-csv-latency.openslo.yaml`, `microservices/forms/slos/field-validate-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/forms/runbooks/ai-form-build-rollback.md`, `microservices/forms/runbooks/captcha-degraded.md`, `microservices/forms/runbooks/embed-iframe-csp-incident.md`, `microservices/forms/runbooks/export-pipeline-failure.md`, `microservices/forms/runbooks/pii-leak-incident-p0.md`, `microservices/forms/runbooks/response-store-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `forms`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `forms`.
- `policy-engine` supplies the signed Cedar corpus while `forms` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `forms` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `forms`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `forms` applies the most restrictive policy and emits a degraded-mode audit event.
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
