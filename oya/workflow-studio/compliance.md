---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-workflow, council-design-system, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0065, ADR-0117, ADR-0123, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145), ADR-0164]
related_artifacts:
  - microservices/workflow-studio/threat-model.md
  - microservices/workflow-studio/dpia.md
  - microservices/workflow-studio/policy/data-residency.md
  - microservices/workflow-studio/policy/editor-isolation.md
  - microservices/workflow-studio/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (workflow-studio µservice)

## Purpose

Canonical control-to-framework mapping for workflow-studio. Tells external auditors (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / EU AI Act notified body / etc.) which control implementation satisfies which framework clause, with pointers to evidence. Continuous-compliance-evidence emission keeps this matrix machine-verifiable.

## Enforced Frameworks

### SOC 2 Type 2 (2017 Trust Services Criteria + 2022 Points of Focus)

| TSC | Control objective | Implementation | Evidence artifact |
|---|---|---|---|
| CC1.1 | Integrity and ethical values | Code-of-conduct + signed-commit policy | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.2 | Board oversight | Council-architecture quarterly review | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI matrix per µservice | `microservices/workflow-studio/CODEOWNERS` |
| CC1.5 | Accountability | Per-µservice SLO + on-call rotation | `PRD.md` §Performance + `incident-response.md` |
| CC2.1 | Communication of information | Status page + tenant comms | `incident-response.md` |
| CC3.1 | Risk identification + assessment | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multi-spectrum review per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; 2-person rule | `policy/editor-isolation.md` |
| CC3.4 | Significant change risk | PR review + LEAN lanes | `branch-protection.yaml` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain emission per save | ADR-0028 + audit-chain µservice |
| CC5.1 | Control activities | LEAN lanes | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar + Citus + RLS + Strict CSP + SRI | `policy/*.cedar` + `policy/editor-isolation.md` |
| CC5.3 | Policy and procedure deployment | Per-µservice runbooks + standards | `docs/standards/*.md` + `runbooks/` |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT via OpenBao | `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`, `policy/ci-scope.cedar` |
| CC6.2 | Authentication + authorization | OIDC + per-tenant SDK API keys + SPIFFE | `threat-model.md` §"Trust boundaries" |
| CC6.3 | Adds / removes access | OpenBao lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Citus + RLS + WS gateway tenant binding | `threat-model.md` T-I-01 |
| CC6.7 | Information transmission + disposal | mTLS in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype; weekly CVE scan | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA WS gateway + per-tenant rate limits + autoscaling | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability metrics + OnCall alerts | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cross-tenant collab SLI | `dashboards/*.json` |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | this changeset |
| CC9.1 | Risk mitigation | Multi-region + DR pair + auto-rollback | `multi-region.md` + ADR-0139 |
| CC9.2 | Vendor risk management | Sub-processor list (including LLM providers) + per-vendor DPA | `legal/sub-processors.md` |

**Privacy Criteria (P1-P8, 2017 TSC, optional):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice; LLM-assist consent UI |
| P2 | Choice + consent | Tenant onboarding consent + per-session LLM-assist opt-in |
| P3 | Collection | SDK PII redactor + `data_class` annotation + canvas data_class markers |
| P4 | Use, retention, disposal | Retention matrix in `data-residency.md`; DSR cascade |
| P5 | Access | Tenant operators read own; DSR access cascade |
| P6 | Disclosure to third parties | Sub-processor list (LLM providers); transfer register |
| P7 | Quality | Spec signature verification + round-trip byte-equality |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS + LLM-assist pack-resident routing | `policy/data-residency.md` |
| A.5.15 | Access control | OIDC + Cedar + Citus + RLS | `threat-model.md` |
| A.5.17 | Authentication information | OpenBao rotation (30d / 90d) | OpenBao audit log |
| A.5.18 | Access rights | RBAC managed via OpenTofu | `iac/terraform/studio-rbac.tf` |
| A.5.23 | Cloud services | OCI HIPAA-eligible for pack-us-healthcare | `policy/data-residency.md` |
| A.5.26 | Response to incidents | Severity-driven runbook | `incident-response.md` + `runbooks/*` |
| A.5.28 | Collection of evidence | Audit-chain Ed25519 emission | ADR-0028 |
| A.5.30 | ICT readiness for BC | Multi-region DR + RPO/RTO | `multi-region.md` |
| A.5.31 | Legal, statutory, regulatory | This document + per-pack overlays | `compliance.md` (this file) |
| A.5.32 | Intellectual property rights | License-policy CI lane + node library signing | `oya-check-license-policy` |
| A.5.33 | Protection of records | Audit-chain immutability + retention | `policy/data-residency.md` |
| A.5.34 | Privacy and protection of PII | DPIA + DSR cascade + Cedar | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access rights | JIT via OpenBao; 2-person rule | OpenBao audit |
| A.8.3 | Information access restriction | Citus + RLS + Cedar + WS gateway tenant binding | `threat-model.md` |
| A.8.4 | Access to source code | CODEOWNERS + branch-protection | `branch-protection.yaml` |
| A.8.5 | Secure authentication | mTLS + OIDC + signed SDK API keys | `threat-model.md` |
| A.8.7 | Protection against malware | Node library Ed25519 signing + WASM SRI | `threat-model.md` T-T-03, T-T-06 |
| A.8.11 | Data masking | SDK redactor for LLM-assist + data_class markers | `threat-model.md` T-I-05 |
| A.8.12 | Data leakage prevention | Strict CSP + Trusted Types + sandboxed iframes (subsequent-to-GA-tier-promotion branding) | `threat-model.md` T-I-02 |
| A.8.15 | Logging | Audit-chain emission | ADR-0028 |
| A.8.16 | Monitoring activities | Studio self-SLI + Grafana | `dashboards/*.json` |
| A.8.20 | Network security | Network policies; Studio → engine/ontology/foundry/tenancy SDKs only | Kubernetes NetworkPolicy review |
| A.8.21 | Network services security | mTLS internal; TLS + WAF public | ingress configuration |
| A.8.23 | Web filtering | WAF + CSP | `iac/helm/*` |
| A.8.25 | Secure development life cycle | LEAN gates + multispectrum review | per-IP evidence |
| A.8.26 | Application security requirements | XSS prevention + signature verification + round-trip byte-equality | `policy/editor-isolation.md` |
| A.8.27 | Secure system architecture | Hexagonal layering per ADR-0103 | `PRD.md` §"Bounded Contexts" |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings`; sanitizers in CI | per-IP acceptance lane |

### GDPR (relevant articles)

| Article | Requirement | Implementation | Evidence |
|---|---|---|---|
| Art. 5 | Principles of processing | Lawful + transparent + minimised + purpose-limited | `dpia.md` §2.4 |
| Art. 6 | Lawful basis | Per purpose in DPIA §2.4 | `dpia.md` |
| Art. 9 | Special categories | pack-us-healthcare BAA; pack-kr explicit consent; LLM-assist consent | `legal/baa-template.md`, `legal/dpa-template.md` |
| Art. 13 + 14 | Information to data subject | Tenant DPA upstream-disclosure clause; LLM-assist transparency UI | `legal/dpa-template.md` |
| Art. 17 | Right to erasure | DSR cascade | `policy/data-residency.md` |
| Art. 22 | Automated decision-making | Studio is authoring tool, not solely-automated-with-legal-effect; LLM-assist is advisory | `dpia.md` R-13 |
| Art. 25 | Privacy by design | This µservice's design satisfies; EDPB Guidelines 4/2019 alignment | `dpia.md` §4 |
| Art. 28 | Processor agreement | DPA template | `legal/dpa-template.md` |
| Art. 30 | Records of processing | ROPA | `legal/ropa.md` |
| Art. 32 | Security of processing | Every threat mitigation contributes | `threat-model.md` |
| Art. 33 | Breach notification (DPA) | 72h notification chain | `incident-response.md` |
| Art. 35 | DPIA | This document + `dpia.md` | `dpia.md` |
| Arts. 44-50 | Transfers | SCC-only; default pack-pinning; LLM-assist pack-resident | `policy/data-residency.md` |

### OWASP ASVS L2 (Application Security Verification)

| ASVS Section | Requirement | Implementation |
|---|---|---|
| V1 (Architecture) | Threat modeling | `threat-model.md` |
| V2 (Authentication) | OIDC + MFA + session token rotation | `threat-model.md` §"Trust boundaries" |
| V3 (Session) | HttpOnly + Secure + SameSite; rotate on auth-state change | Studio cookie configuration |
| V4 (Access Control) | Default-deny Cedar + per-tenant scope | `policy/tenant-scope.cedar` |
| V5 (Validation) | Server-side schema validation; canonical JSON-Schema | `threat-model.md` T-I-03 |
| V7 (Errors + Logging) | No sensitive data in error responses; audit-chain emission | `threat-model.md` §"R-family" |
| V8 (Data Protection) | Data classification + retention bounds | `dpia.md` + `policy/data-residency.md` |
| V9 (Communications) | mTLS internal; TLS 1.3 public | ingress configuration |
| V11 (Business Logic) | Per-seat license-gate + round-trip byte-equality | `threat-model.md` T-T-05, AC-15 |
| V12 (Files + Resources) | SRI hashes on WASM chunks; signed node libraries | `threat-model.md` T-T-06, T-T-03 |
| V13 (API) | OpenAPI 3.1; gRPC proto; AsyncAPI 3.0 | `contracts/*` |
| V14 (Configuration) | Strict CSP + Trusted Types + WAF | `policy/editor-isolation.md` |

### EU AI Act 2024 (conditional; when LLM-assist used in regulated domain)

| AI Act Article | Requirement | Implementation |
|---|---|---|
| Art. 9 | Risk management system | LLM-assist conformity assessment per `legal/ai-act-conformity.md` |
| Art. 10 | Data and data governance | LLM-assist prompts + completions retained 90d; data quality via schema validation |
| Art. 12 | Record-keeping | Audit-chain seal per LLM-assist invocation; retention ≥ 6mo when in high-risk context |
| Art. 13 | Transparency + provision of information | LLM-assist transparency UI in editor; tenant onboarding disclosure |
| Art. 14 | Human oversight | User explicit-accept of LLM-drafted spec before save (never auto-submit) |
| Art. 15 | Accuracy, robustness, cybersecurity | Schema validation + signature verification + tenant approval gate |

## Per-Pack Overlay Sections

### pack-kr (KR-ISMS-P + KR PIPA + KR 전자문서법)

KR PIPA Art. 29 (technical safeguards) — cross-mapped to Studio mitigations:

| PIPA safeguard | Studio mitigation |
|---|---|
| Access control | OIDC + Cedar + Citus + RLS + WS gateway tenant binding |
| Encryption (transit) | mTLS internal; TLS public ingress |
| Encryption (at rest) | KMS-SSE for Postgres + Valkey AOF + object-storage |
| Integrity verification | Round-trip byte-equality + audit-chain Merkle |
| Audit log retention ≥ 1y | 3y default for KR-FSS sector |
| IDS / IPS | WAF + network policies |
| Vulnerability management | `cargo deny` + Trivy + Grype |
| Mobile / remote access | mTLS + OIDC + MFA |
| User account management | OpenBao lifecycle + per-seat Cedar |
| Logging | OTel + audit-chain |
| Patch management | Helm + ArgoCD declarative |
| Incident response | Severity-classified runbooks |

KR 전자문서법 (Electronic Document Act):
- Art. 5 (integrity preservation): Ed25519 audit-chain seal satisfies.
- Art. 6 (long-term preservation): Audit-chain immutability + retention satisfy.
- Art. 7 (admissibility): Audit-chain Merkle proof is admissible evidence.

### pack-us-healthcare (HIPAA)

| HIPAA section | Requirement | Studio implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) | Risk analysis | This document + `dpia.md` + `threat-model.md` |
| §164.308(a)(4)(ii)(B) | Access authorization | Cedar policy + 2-person rule |
| §164.310(a)(1) | Facility access controls | Inherited from cloud-k8s + OCI HIPAA-eligible |
| §164.312(a)(1) | Access control (unique user ID) | OIDC + per-user identity |
| §164.312(a)(2)(i) | Emergency access | JIT elevation procedure |
| §164.312(a)(2)(ii) | Automatic logoff | Session timeout |
| §164.312(b) | Audit controls | Audit-chain emission per save + license-gate |
| §164.312(c)(1) | Integrity | Round-trip byte-equality + audit-chain Merkle |
| §164.312(d) | Person or entity authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) | Transmission security | mTLS + TLS |
| §164.316(b)(2) | Documentation retention (6y) | Repo retention + audit-chain |

LLM-assist provider for pack-us-healthcare must be HIPAA BAA-eligible.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + DORA + AI Act)

- GDPR Art. 32 cross-mapped (above).
- EDPB Guidelines 4/2019 (Art. 25): every mitigation maps to TOM.
- NIS2 (2022/2555): 24h + 72h + 1mo reporting timelines when thresholds crossed.
- eIDAS 910/2014 Art. 26: Ed25519 audit-chain seals satisfy AdES.
- DORA (2022/2554): pack-eu financial-services tenants get DORA-aligned BCDR.
- EU AI Act 2024: LLM-assist conformity per §"EU AI Act 2024" above.

### pack-jp (APPI)

| APPI Article | Requirement | Studio implementation |
|---|---|---|
| Art. 17 | Purpose of use declaration | DPA + tenant onboarding |
| Art. 18 | Consent for sensitive data | Tenant DPA |
| Art. 20 | Security control measures | Threat-model mitigations |
| Art. 21 | Cross-border transfer | pack-jp residency; LLM-assist routes JP-resident |
| Art. 23 | Joint use disclosure | Tenant DPA upstream clause |
| Art. 24 | Provision to third party | Sub-processor list |
| Art. 26-2 | Breach notification | `incident-response.md` |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/workflow-studio-overlay.md` with full citations.

## Continuous-Compliance Evidence

The `oya-governance-compliance-evidence-recency` lane validates that each control's evidence artifact:
- Exists at its declared path.
- Was last modified within freshness window (annual default; quarterly for sensitive controls).
- Has a valid Ed25519 signature where required.

Annual external audit reads this file + evidence artifacts cited above.

## Re-review Triggers

- Annually (Q2 each year).
- On any pack activation.
- On any enforced-framework version update.
- On any sub-processor change (including LLM provider).
- Post-incident (Sev-1 or Sev-2).
- On any change to Studio's processing scope.
- EU AI Act enforcement milestone.

## References

- ADR-0028 (Bominal): Audit chain.
- ADR-0065: Leptos for browser UI.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- ADR-0164 (Bominal): Workflow canonical spec format.
- `microservices/workflow-studio/threat-model.md`.
- `microservices/workflow-studio/dpia.md`.
- `microservices/workflow-studio/policy/data-residency.md`.
- `microservices/workflow-studio/policy/editor-isolation.md`.
- `microservices/workflow-studio/incident-response.md`.
- SOC 2 Type 2 (2017 TSC + 2022 PoF).
- ISO 27001:2022 Annex A.
- GDPR EUR-Lex 2016/679.
- KR PIPA + 전자문서법 + KR-ISMS-P.
- HIPAA 45 CFR Parts 160 + 164.
- APPI 2003 (改正 2022).
- LGPD 2018.
- DPDPA 2023.
- DORA 2022/2554.
- NIS2 2022/2555.
- eIDAS 910/2014.
- EU AI Act 2024.
- OWASP ASVS v4.0.

---



## §day-one-cert-readiness
This anchor is closed for `workflow-studio` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `workflow-studio` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +14 more.
- Example: `t0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `t0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `workflow-studio` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `t0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `workflow-studio` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`, `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`; +18 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `t0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.workflow-studio.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `workflow-studio` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `t0-suggest` touches those data classes.
- Signal sources: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`; +11 more.
- Example event class: `oya.workflow.studio.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `workflow-studio` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.workflow-studio.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `t0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `t0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `workflow-studio` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`, `workflow_studio.workflow_studio`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `workflow_studio.workflow_studio` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `workflow-studio` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`; +12 more.
- Example: `t0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.workflow-studio` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/workflow-studio/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.
- Example: `t0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `workflow-studio` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`, `microservices/workflow-studio/iac/helm/visual-canvas-rest/Chart.yaml`, `microservices/workflow-studio/iac/helm/visual-canvas-rest/templates/deployment.yaml`, `microservices/workflow-studio/iac/helm/visual-canvas-rest/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `t0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `workflow-studio` is in annual full-scope pentest and every major `t0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`, `microservices/workflow-studio/iac/helm/visual-canvas-rest/Chart.yaml`, `microservices/workflow-studio/iac/helm/visual-canvas-rest/templates/deployment.yaml`, `microservices/workflow-studio/iac/helm/visual-canvas-rest/templates/hpa.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `workflow-studio` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `workflow-studio` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `t0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `workflow-studio` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-adapter-valkey.yaml`, `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-domain.yaml`, `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-worker.yaml`, `microservices/workflow-studio/catalog/oya-workflow-studio-dsl-emitter-domain.yaml`, `microservices/workflow-studio/catalog/oya-workflow-studio-dsl-loader-domain.yaml`, `microservices/workflow-studio/catalog/oya-workflow-studio-jurisdiction-overlay-renderer-domain.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `t0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `workflow-studio` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `t0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `t0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `workflow-studio` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `workflow_studio.workflow_studio`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `t0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `workflow-studio`; owner `axis-workflow-studio`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `workflow-studio`.
- Capability records cited: `microservices/workflow-studio/capabilities/T0-suggest.yaml`, `microservices/workflow-studio/capabilities/T1-assist.yaml`, `microservices/workflow-studio/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar/policy artifacts cited: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +5 more.
- Runbook/IaC evidence: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`, `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`, `microservices/workflow-studio/contracts/proto/workflow-studio.proto`.
- Cedar binding: `microservices/workflow-studio/policy/auditor-scope.cedar`, `microservices/workflow-studio/policy/ci-scope.cedar`, `microservices/workflow-studio/policy/data-residency.md`, `microservices/workflow-studio/policy/editor-isolation.md`, `microservices/workflow-studio/policy/public-read.cedar`, `microservices/workflow-studio/policy/tenant-scope.cedar`.
- State/event binding: `workflow_studio.workflow_studio`.
- Capability binding: `t0-suggest`, `t1-assist`, `t2-auto`.
- SLO binding: `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-merge-latency.openslo.yaml`, `microservices/workflow-studio/slos/collab-crdt-no-silent-loss.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-availability.openslo.yaml`, `microservices/workflow-studio/slos/editor-rest-latency.openslo.yaml`, `microservices/workflow-studio/slos/editor-tti-cdn-availability.openslo.yaml`; +1 more.
- Runbook binding: `microservices/workflow-studio/runbooks/canvas-perf-regression-debug.md`, `microservices/workflow-studio/runbooks/canvas-perf-regression.md`, `microservices/workflow-studio/runbooks/collab-conflict-resolution.md`, `microservices/workflow-studio/runbooks/copilot-degraded-fallback.md`, `microservices/workflow-studio/runbooks/crdt-merge-conflict.md`, `microservices/workflow-studio/runbooks/presence-disconnect.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `workflow-studio`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `workflow-studio`.
- `policy-engine` supplies the signed Cedar corpus while `workflow-studio` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `workflow-studio` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `workflow-studio`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `workflow-studio` applies the most restrictive policy and emits a degraded-mode audit event.
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
