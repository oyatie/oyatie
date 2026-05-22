---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-social, council-architecture, ops-compliance
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0123, ADR-0135, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/social/threat-model.md
  - microservices/social/dpia.md
  - microservices/social/policy/dual-context-isolation.md
  - microservices/social/policy/data-residency.md
  - microservices/social/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (social µservice)

## Purpose

Canonical control-to-framework mapping for the social µservice. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / EU DSA Coordinator / EU AI Act notified body) exactly which control implementation satisfies which framework clause, with pointers to the evidence artifact.

## Enforced Frameworks (every pack)

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | Integrity + ethical values | Code-of-conduct + signed commits | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.5 | Accountability for performance | Per-µservice SLO + on-call | `slos/social-feed-render-latency.openslo.yaml` + `incident-response.md` |
| CC3.1 | Risk identification | Threat model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to objectives | Multi-spectrum review per ADR + IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; four-eyes disclosure; sybil-detector | `policy/dual-context-isolation.md` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLO | `/registry/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain on every state transition | ADR-0028 + audit-chain µservice |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT | `policy/*.cedar` |
| CC6.2 | Authn + authz | Per-tenant API keys + SPIFFE | `policy/tenant-scope.cedar` |
| CC6.3 | Access lifecycle | OpenBao adds/removes + audit | OpenBao audit log |
| CC6.6 | Logical access controls | Postgres RLS + Cedar + reserved tenants | `threat-model.md` T-I-01 mitigation |
| CC6.7 | Transmission + disposal | mTLS in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §DSR |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype; weekly CVE; ImageMagick + ffmpeg CVE scan | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA Postgres + per-tenant rate limits + HPA | `capacity-model.md` |
| CC7.2 | Monitoring inputs | Self-observability via observability µservice | `slos/` + `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts + sybil + trending poisoning | OpenSLO manifests |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates | observability promotion gate per ADR-0139 |
| CC9.1 | Risk mitigation | Multi-region + DR + automated rollback | `multi-region.md` |
| CC9.2 | Vendor risk | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` (Slice B) |

**Privacy Criteria (P1–P8):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice + EU AI Act Art. 50 transparency label |
| P2 | Choice + consent | OpenBao tenant-resolver onboarding consent + age-gate + child-consent flow |
| P3 | Collection | OTel SDK PII redactor + `data_class` annotation |
| P4 | Use, retention, disposal | Retention matrix in `policy/data-residency.md`; DSR cascade |
| P5 | Access | Tenant operators read own data |
| P6 | Disclosure to third parties | Sub-processor list + transfer register + federation peer allowlist |
| P7 | Quality | Audit-chain integrity + four-eyes disclosure + appeal workflow |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Threat-model review cadence; threat-intel feeds | `threat-model.md` |
| A.5.10 | Acceptable use | Internal AUP + onboarding | `docs/standards/onboarding.md` |
| A.5.14 | Info transfer | mTLS + KMS + signed audit-chain + signed ActivityPub federation | `threat-model.md` Trust Boundary 3 + 9 |
| A.5.15 | Access control | Cedar fragments + OIDC + MFA | `policy/*.cedar` |
| A.5.17 | Authentication info | OpenBao secret lifecycle + rotation | OpenBao audit log |
| A.5.18 | Access rights | Per-resource Cedar + four-eyes for disclosure | `policy/tenant-scope.cedar` |
| A.5.23 | Cloud-service security | Multi-region + DR posture | `multi-region.md` |
| A.5.26 | Incident response | Severity-classified IR; postmortems | `incident-response.md` |
| A.5.30 | ICT readiness for BCDR | DR pair + RPO/RTO targets | `multi-region.md` |
| A.5.31 | Legal + statutory | Per-pack regulatory cross-mapping below | this doc |
| A.5.34 | Privacy + PII protection | Data-class taxonomy + DSR cascade + four-eyes | `policy/data-residency.md` §DSR |
| A.8.2 | Privileged access rights | JIT elevation; two-person rule for admin ops | OpenBao audit |
| A.8.3 | Info access restriction | Cedar + RLS + per-tenant key bindings | `threat-model.md` T-S-01 mitigation |
| A.8.5 | Secure authentication | OIDC + MFA + OAuth 2.1; mTLS internal | `policy/tenant-scope.cedar` |
| A.8.7 | Protection against malware | OPSWAT / ClamAV media scan + quarantine; ImageMagick + ffmpeg sandboxed (gVisor / Kata) | `threat-model.md` T-E-05 + media-malware runbook |
| A.8.11 | Data masking | Span redactor; media preview redactor; search-result Cedar filter | `policy/redaction-phi.md` (pack-us-healthcare) |
| A.8.12 | Data leakage prevention | DLP via PII detectors + cardinality limits + LEAN coverage; federation-personal-tier-refused | `threat-model.md` T-I-08 mitigation |
| A.8.20 | Networks security | Service mesh + mTLS + NetworkPolicy | k8s NetworkPolicy review |
| A.8.21 | Network services | TLS termination + WAF + DDoS | ingress configuration |
| A.8.23 | Web filtering | n/a (server-side service) | – |
| A.8.25 | Secure development lifecycle | LEAN gates + multispectrum review | `evidence/multispectrum/` |
| A.8.27 | Application security | OWASP API Top 10 mitigations; cargo audit | `threat-model.md` |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings` + `cargo deny` | CI lanes |
| A.8.32 | Change management | PR + LEAN + branch-protection | branch-protection.yaml |
| A.8.34 | Audit findings remediation | Audit-finding tracker per engagement | ops-compliance |

### GDPR

| Article | Implementation | Evidence |
|---|---|---|
| Art. 5 (principles) | Data-class taxonomy + minimisation + retention | `policy/data-residency.md` |
| Art. 6 (lawful basis) | Per-class lawful-basis declared in `dpia.md` §2.2 | `dpia.md` |
| Art. 7 (consent) | Tenant onboarding + per-user signup consent | `legal/dpa-template.md` (Slice B) |
| Art. 8 (child consent) | `age-verification` BC + per-pack threshold | `policy/age-gate.md` (Slice B) |
| Art. 9 (special-category) | Pack-us-healthcare BAA + KR PIPA Art. 23 consent | `legal/baa-template.md` |
| Art. 13/14 (transparency) | Tenant onboarding notice; joint-controllership clause; EU AI Act Art. 50 label | `legal/dpa-template.md` |
| Art. 17 (erasure) | DSR cascade | `policy/data-residency.md` §DSR |
| Art. 22 (automated decisions) | Algorithmic-ranking transparency + appeal workflow; opt-out to chronological | `capabilities/T2-auto.yaml` |
| Art. 25 (privacy-by-design) | Dual-context invariant; redactor; Cedar | `policy/dual-context-isolation.md` |
| Art. 28 (processor) | Per-tenant DPA | `legal/dpa-template.md` |
| Art. 30 (records of processing) | Audit-chain ledger | audit-chain µservice |
| Art. 32 (security) | Every mitigation in `threat-model.md` | `threat-model.md` |
| Art. 33 (breach notification) | IR playbook; 72h GDPR clock | `incident-response.md` |
| Art. 35 (DPIA) | This DPIA satisfies | `dpia.md` |
| Art. 44–50 (transfers) | Pack-pinning; SCC required for cross-border; federation peer allowlist | `policy/data-residency.md` |

### EU DSA (Digital Services Act 2065/2022)

| Article | Implementation | Evidence |
|---|---|---|
| Art. 14 (terms-of-service transparency) | Per-tenant ToS + plain-language summary | tenant onboarding |
| Art. 16 (notice-and-action) | `AbuseReportFiled` Workflow event + `runbooks/abuse-report-backlog-drain.md` | content-moderation BC |
| Art. 17 (Statement of Reasons) | Every moderation verdict emits a Statement of Reasons + audit-chain seal | `capabilities/T2-auto.yaml` evidence pipeline |
| Art. 20 (internal complaint-handling) | `Appeal` workflow; ≤ 7d resolution | content-moderation BC |
| Art. 23 (out-of-court dispute settlement) | Pointer to certified dispute body; tenant-pack overlay | `legal/dsa-overlay-eu.md` (Slice B) |
| Art. 24 (transparency reports) | Per-tenant transparency log; quarterly export | `compliance.md` Continuous Evidence |
| Art. 27 (recommender system transparency) | Algorithmic feed explanation API + user-controllable chronological override | `capabilities/T2-auto.yaml` |
| Art. 28 (online protection of minors) | `age-verification` BC + minor-protection policy | per pack |

### EU AI Act 2024/1689

| Article | Implementation | Evidence |
|---|---|---|
| Art. 9 (risk-management system) | Per-classifier risk register + mitigation tracker | `capabilities/T2-auto.yaml` |
| Art. 10 (data + governance) | Training-dataset SHA + bias-audit + reference-set eval | foundry-runtime evidence pipeline |
| Art. 11 (technical documentation) | Per-classifier model card + ADR-SOC-0003 | foundry-runtime |
| Art. 13 (transparency to users) | Per-verdict `eu_ai_act_label: ai_generated_assessment` + appeal pointer | `capabilities/T2-auto.yaml` |
| Art. 14 (human oversight) | Manual reviewer in appeal workflow; mandatory human review for verdicts above confidence threshold | content-moderation BC |
| Art. 15 (accuracy + robustness + cybersecurity) | Per-release reference-set eval + adversarial robustness eval | foundry-runtime |
| Art. 50 (transparency obligation) | UI label "AI-assessed" on every classifier verdict | client SDK |
| Art. 52 (codes of conduct) | Voluntary code-of-conduct alignment | ops-compliance |

## Per-Pack Overlays

### pack-kr

| KR clause | Implementation |
|---|---|
| KR PIPA Art. 8 (child consent) | `age-verification` minor flow + parental consent attestation |
| KR PIPA Art. 15 (collection consent) | Tenant onboarding + per-user signup |
| KR PIPA Art. 17 (cross-border consent) | Pack-pinning; cross-border requires explicit consent |
| KR PIPA Art. 22-2 (sensitive consent) | pack-kr sensitive profiles require additional consent flow |
| KR PIPA Art. 23 (sensitive data) | Encryption + Cedar entitlement + four-eyes for disclosure |
| KR PIPA Art. 28 (processor) | Tenant DPA |
| KR PIPA Art. 29 (technical safeguards) | All `threat-model.md` mitigations map to Art. 29 controls |
| KR PIPA Art. 29-2 (KR-specific) | Audit log retention ≥ 1 year |
| KR-ISMS-P §2.5 (personnel) | Two-person rule + JIT elevation |
| KR-ISMS-P §2.7 (access control) | Cedar |
| KR 정보통신망법 §49 (intercept) | Server-side admin reads only via four-eyes |
| KR 청소년 보호법 | Minor-protection routing in `age-verification` |
| KR 전자문서법 Art. 5 (integrity) | Audit-chain Ed25519 seal |

### pack-us-healthcare

| HIPAA clause | Implementation |
|---|---|
| §164.308(a)(1)(ii)(A) (risk analysis) | DPIA + threat-model |
| §164.308(a)(4)(ii)(B) (access authorization) | Cedar + OIDC + MFA |
| §164.310 (physical safeguards) | OCI-managed datacenter (BAA-eligible) |
| §164.312(a)(1) (access control) | Cedar + Postgres RLS + four-eyes |
| §164.312(b) (audit controls) | Audit-chain ≥ 6y retention |
| §164.312(c)(1) (integrity) | content-hash + audit-chain |
| §164.312(e)(1) (transmission security) | mTLS + KMS |
| §164.502(b) (minimum-necessary) | Media preview + search redaction |
| §164.514 (de-identification) | PHI redactor pre-index |
| BAA template | `legal/baa-template.md` |
| US COPPA 15 USC §6501 | `age-verification` BC; minor-account flow |

### pack-eu

| Clause | Implementation |
|---|---|
| GDPR Art. 8 + 25 + 32 | Child consent + dual-context invariant; mitigations table |
| GDPR Art. 22 + 35 | Algorithmic ranking transparency + DPIA |
| ePrivacy Directive Art. 5(3) | Confidentiality via Cedar + RLS |
| NIS2 2022/2555 (when thresholds engaged) | IR playbook 24h/72h/1mo timelines |
| eIDAS 910/2014 | Ed25519 audit-chain seals = AdES |
| EU DSA Arts. 14, 16, 17, 20, 23, 24, 27, 28 | Notice-and-action + Statement of Reasons + appeal + transparency report + minor protection |
| EU AI Act Arts. 9, 10, 11, 13, 14, 15, 50, 52 | Risk-management + data governance + transparency + human oversight + accuracy + per-verdict label |
| UK Online Safety Act 2023 (UK-located tenants) | Ofcom illegal-content duty; safety-by-design report |

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack-overlay `regional-packs/<pack>/social-compliance-overlay.md`.

## Continuous Compliance Evidence

CI lane `oya-governance-compliance-evidence-recency --microservice social` evaluates every 24h:

- All policy/*.cedar files lint clean.
- All Helm charts pass `helm lint`.
- All OpenSLO manifests pass schema validation.
- All runbooks have a `last_drill_date` within 90 days.
- All threat-model rows have a re-review date within 90 days for residual ≥ M.
- All DPIA rows have a re-review date within 365 days.
- Per-tenant DPA + BAA signed status reflected in compliance dashboard.
- Per-classifier-version EU AI Act reference-set eval recorded; per-release model card sealed.
- EU DSA transparency report per-tenant exportable (quarterly).
- Pack-aware age-gate test exit 0 per pack.

Output: `microservices/social/evidence/compliance-evidence-<unix_ts>.json`.

## References

- `microservices/social/threat-model.md`.
- `microservices/social/dpia.md`.
- `microservices/social/policy/dual-context-isolation.md`.
- `microservices/social/policy/data-residency.md`.
- `microservices/observability/compliance.md` (shape reference).
- `microservices/messenger/compliance.md` (sibling reference; dual-context posture).
- ADR-0028 (Bominal) + ADR-0008 + ADR-0135 + ADR-0139 + ADR-0131 + ADR-0140.
- EU DSA 2065/2022; EU AI Act 2024/1689; UK Online Safety Act 2023.

---



## §day-one-cert-readiness
This anchor is closed for `social` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `social` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +22 more.
- Example: `composer-suggest-and-hashtag-completion` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more without changing domain code.
- Data classes under pack control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `composer-suggest-and-hashtag-completion` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `social` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `composer-suggest-and-hashtag-completion` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `social` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`, `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`; +21 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `composer-suggest-and-hashtag-completion` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.social.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `social` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `composer-suggest-and-hashtag-completion` touches those data classes.
- Signal sources: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +22 more.
- Example event class: `oya.social.composer.suggest.and.hashtag.completion.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §ml-model-lifecycle
This anchor is closed for `social` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `True` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `social.composer-suggest-and-hashtag-completion` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `composer-suggest-and-hashtag-completion` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §detection-fairness-audit
This anchor is closed for `social` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `social` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `social` never stores protected attributes solely to make a product feature easier.
- Example: `composer-suggest-and-hashtag-completion` abuse/risk score challenge rate is compared across locale, accessibility profile, age tier, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `social` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.social.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `composer-suggest-and-hashtag-completion` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `composer-suggest-and-hashtag-completion` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `social` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`, `social.social`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `social.social` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `social` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +18 more.
- Example: `composer-suggest-and-hashtag-completion` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.social` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/social/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.
- Example: `composer-suggest-and-hashtag-completion` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `social` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`, `microservices/social/iac/ech-config.yaml`, `microservices/social/iac/edge-waf.yaml`, `microservices/social/iac/helm/social/Chart.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `composer-suggest-and-hashtag-completion` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `social` is in annual full-scope pentest and every major `composer-suggest-and-hashtag-completion` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`, `microservices/social/iac/ech-config.yaml`, `microservices/social/iac/edge-waf.yaml`, `microservices/social/iac/helm/social/Chart.yaml`; +21 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `social` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `social` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `composer-suggest-and-hashtag-completion` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `social` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/social/catalog/oya-social-app.yaml`, `microservices/social/catalog/oya-social-content-moderation-adapter-clamav.yaml`, `microservices/social/catalog/oya-social-content-moderation-adapter-opswat.yaml`, `microservices/social/catalog/oya-social-content-moderation-kernel.yaml`, `microservices/social/catalog/oya-social-csam-classifier-adapter-photodna.yaml`, `microservices/social/catalog/oya-social-dsa-transparency-worker.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `composer-suggest-and-hashtag-completion` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `social` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `composer-suggest-and-hashtag-completion` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `composer-suggest-and-hashtag-completion` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `social` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- State/event surfaces carrying classification: `social.social`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `composer-suggest-and-hashtag-completion` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `social`; owner `axis-social`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `social`.
- Capability records cited: `microservices/social/capabilities/T0-suggest.yaml`, `microservices/social/capabilities/T1-assist.yaml`, `microservices/social/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar/policy artifacts cited: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +18 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/social/contracts/asyncapi/social-events.yaml`, `microservices/social/contracts/openapi/social.yaml`, `microservices/social/contracts/proto/social.proto`.
- Cedar binding: `microservices/social/policy/abuse-defence.cedar`, `microservices/social/policy/auditor-scope.cedar`, `microservices/social/policy/ci-scope.cedar`, `microservices/social/policy/content-policy.cedar`, `microservices/social/policy/data-residency.md`, `microservices/social/policy/dm-scope.cedar`; +6 more.
- State/event binding: `social.social`.
- Capability binding: `composer-suggest-and-hashtag-completion`, `caption-and-alt-text-and-summary`, `content-moderation-classifier-and-ranking-and-ads-stub`.
- SLO binding: `microservices/social/slos/content-policy-enforcement-correctness.openslo.yaml`, `microservices/social/slos/csam-classifier-latency.openslo.yaml`, `microservices/social/slos/feed-render-latency.openslo.yaml`, `microservices/social/slos/follow-action-latency.openslo.yaml`, `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml`, `microservices/social/slos/moderation-classifier-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/social/runbooks/abuse-report-backlog-drain.md`, `microservices/social/runbooks/content-moderation-rollback.md`, `microservices/social/runbooks/coordinated-inauthentic-behavior-response.md`, `microservices/social/runbooks/csam-detect-and-ncmec-report.md`, `microservices/social/runbooks/dsa-transparency-report-generation.md`, `microservices/social/runbooks/federation-bridge-degraded.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `social`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `social`.
- `policy-engine` supplies the signed Cedar corpus while `social` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `social` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `social`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `social` applies the most restrictive policy and emits a degraded-mode audit event.
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
