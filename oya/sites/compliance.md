---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140 (retired per ADR-0145), ADR-SITES-0006]
doc_status: published
---

# Compliance Matrix — sites µservice

## Purpose

Enumerate compliance frameworks engaged by sites, the controls
satisfied, and where each control is evidenced (per artifact, lane, or
runbook).

## Frameworks engaged

### Globally enforced

| Framework | Scope | Mapping |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x–CC9.x | §"SOC 2 Mapping" below |
| ISO 27001:2022 | Annex A.5–A.8 | §"ISO 27001 Mapping" below |
| GDPR | Arts. 5, 6, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 | §"GDPR Mapping" below |
| WCAG 2.2 AA + ATAG 2.0 | accessibility | §"Accessibility Mapping" below |
| OWASP ASVS v4 | application security verification | §"OWASP Mapping" below |
| W3C Subresource Integrity | published asset integrity | LEAN `oya-check-sri-coverage` |
| NIST SSDF (SP 800-218) | secure software development | per Foundry pipeline |
| SLSA L3 | supply-chain integrity | published artifacts signed |
| CIS Kubernetes Benchmark | container hardening | IaC inherits from cloud-k8s |

### Pack-overlays

| Pack | Frameworks |
|---|---|
| pack-kr | KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2/33 + ISMS-P §2.1-2.12 + 전자문서법 Arts. 5/6/7 |
| pack-us-healthcare | HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502/§164.504/§164.512/§164.514 + ADA Title III + Section 508 (patient portals) + CCPA / CMIA / NY SHIELD |
| pack-eu | GDPR + EDPB Guidelines 4/2019 + 9/2022 + EU AI Act Art. 50 + Annex III §3 + EU DSA Arts. 14/27 + NIS2 + eIDAS 910/2014 + ePrivacy 2002/58/EC Art. 5(3) |
| pack-jp | APPI Arts. 17/18/20/21/23/24/26-2/27 |
| pack-sg | PDPA 2012 Parts III/IV/VI + MAS Notice 644 (financial-tenants) |
| pack-au | Privacy Act 1988 APP 1-13 + APRA-CPS 234 (financial-tenants) |
| pack-in | DPDPA 2023 §6-16 + RBI Master Direction on IT Outsourcing (financial-tenants) |
| pack-br | LGPD Arts. 6/7/11/14/18/33/38/46/48 + BACEN Res. 4.893/2021 (financial-tenants) |
| pack-ae | UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15/22/23 |
| pack-ksa | KSA PDPL Royal Decree M/19/2021 Arts. 4-9 + SAMA Cybersecurity Framework 2017 (financial-tenants) |

## SOC 2 Mapping

| TSC | Control | Sites evidence |
|---|---|---|
| CC1.1 | Demonstrates commitment to integrity | council-architecture + ops-security sign-off on threat-model + DPIA |
| CC2.1 | Communicates information | This compliance.md + PRD + runbooks |
| CC3.1 | Specifies risk objectives | threat-model.md + dpia.md |
| CC5.1 | Selects + develops control activities | LEAN CI lanes + Cedar policies |
| CC6.1 | Logical access — restricts | per-tenant RLS + tenant-DEK + Cedar policies |
| CC6.2 | Authenticates | OIDC + MFA + per-tenant API keys |
| CC6.3 | Authorises | Cedar policies (`tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`) |
| CC6.6 | Restricts physical access | inherited from cloud-k8s + cloud-secrets |
| CC6.7 | Restricts info flow | type-narrowed projections + LEAN checks |
| CC6.8 | Prevents unauthorized software | branch-protection + signed commits + signed artifacts + SLSA L3 + W3C SRI |
| CC7.1 | Detects security events | observability alerts + audit-chain |
| CC7.2 | Monitors system components | SLO + burn-rate dashboards |
| CC7.4 | Responds to incidents | incident-response.md + runbooks |
| CC7.5 | Recovers from incidents | DR drills + backup retention; runbooks/publish-pipeline-rollback.md |
| CC8.1 | Manages changes | ADR-0110 ChangeSet + ADR-0139 SLO-gated promotion |
| CC9.1 | Identifies risks | residual-risk acceptance in threat-model.md |

## ISO 27001:2022 Mapping

| Annex A Control | Sites evidence |
|---|---|
| A.5.7 (threat intelligence) | threat-model.md re-review triggers |
| A.5.10 (acceptable use of info) | policy/editor-isolation.md |
| A.5.14 (information transfer) | policy/data-residency.md |
| A.5.15 (access control) | Cedar policies + RLS |
| A.5.17 (authentication info) | OpenBao + per-tenant DEK rotation |
| A.5.23 (cloud service usage) | ADR-0117 |
| A.5.30 (ICT continuity) | multi-region.md |
| A.5.31 (legal + statutory) | This compliance.md per-pack overlays |
| A.5.34 (privacy + PII) | dpia.md + policy/* |
| A.8.2 (privileged access) | OpenBao JIT + 2-person rule |
| A.8.5 (secure authentication) | OIDC + MFA |
| A.8.11 (data masking) | redaction in published projection + WCAG-AA correctness lane |
| A.8.12 (data leakage prevention) | LEAN checks + DLP scan on publish |
| A.8.15 (logging) | observability + audit-chain |
| A.8.20 (network security) | mesh mTLS + NetworkPolicy |
| A.8.23 (web filtering) | WAF at ingress; rate-limit on anonymous reads |
| A.8.25 (secure SDLC) | LEAN gates + ADR-0139 |
| A.8.26 (application security) | per-microservice security artifacts |
| A.8.27 (secure system architecture) | ADR-0056 + ADR-0105 + ADR-0131 |
| A.8.28 (secure coding) | LEAN check + cargo fuzz |
| A.8.32 (change management) | ADR-0110 + branch-protection |
| A.8.34 (audit + protection) | audit-chain immutability + 2-person rule |

## GDPR Mapping

| Article | Sites evidence |
|---|---|
| Art. 5(1)(a) lawfulness | per-purpose lawful basis in dpia.md §2.2 |
| Art. 5(1)(b) purpose limitation | dpia.md §2.4 |
| Art. 5(1)(c) data minimisation | type-narrowed published projection + analytics hash-bucket |
| Art. 5(1)(d) accuracy | tenant-edit UX + audit history |
| Art. 5(1)(e) storage limitation | retention per pack |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK + Ed25519 audit-chain + W3C SRI |
| Art. 6(1) lawful basis | dpia.md §2.2 |
| Art. 13/14 transparency | tenant DPA template + consent banner (where required by ePrivacy) |
| Art. 17 right-to-erasure | DSR cascade orchestrator in page-usecase + legal-hold reconciliation |
| Art. 20 portability | site-export endpoint per FR-portability |
| Art. 22 automated decision | T2 AI-page-build does not make legal-effect decisions; HR/legal/medical overlay REFUSED per ADR-SITES-0006 |
| Art. 25 by design + default | type-system separation + Cedar policy |
| Art. 28 processor agreement | DPA template |
| Art. 30 records of processing | RoPA template |
| Art. 32 security of processing | every STRIDE / LINDDUN mitigation |
| Art. 33 breach notification | incident-response.md 72h chain |
| Art. 35 DPIA | dpia.md |
| Arts. 44-50 transfers | SCC + transfer register + multi-region.md |

## Accessibility Mapping (WCAG 2.2 AA + ATAG 2.0)

| Criterion | Sites evidence |
|---|---|
| WCAG 2.2 1.1.1 Non-text Content | alt-text required at publish; LEAN refuse |
| WCAG 2.2 1.3.1 Info and Relationships | heading-order LEAN check |
| WCAG 2.2 1.4.3 Contrast (Minimum) | 4.5:1 contrast check via LightningCSS + theme validator |
| WCAG 2.2 1.4.10 Reflow | responsive layout default themes |
| WCAG 2.2 2.1.1 Keyboard | all interactive blocks keyboard-navigable |
| WCAG 2.2 2.4.4 Link Purpose | link text required + LEAN |
| WCAG 2.2 2.5.8 Target Size (Minimum) | per ADR-SITES-0007 image-block min 44×44 tap target |
| WCAG 2.2 3.1.1 Language of Page | `lang` attribute required at publish |
| WCAG 2.2 3.3.7 Redundant Entry | form-block via forms µservice |
| WCAG 2.2 4.1.2 Name, Role, Value | ARIA on dynamic blocks |
| ATAG 2.0 (editor accessibility) | block editor accessible; alt-text prompt on image insert |

## OWASP ASVS v4 Mapping

| Section | Coverage |
|---|---|
| V1 Architecture | ADR-0056 + ADR-0105 + per-µservice flat layout |
| V2 Authentication | OIDC + MFA + per-tenant API key |
| V3 Session management | per-tenant session token; salt-rotation |
| V4 Access control | Cedar + RLS |
| V5 Validation, sanitization, encoding | URL percent-encoding per RFC 3986; HTML output encoding; SVG sanitisation; LightningCSS scoped CSS |
| V7 Error handling + logging | structured logging + audit-chain |
| V8 Data protection | tenant-DEK + TLS 1.3 |
| V9 Communication | mesh mTLS + WAF |
| V10 Malicious code | image-pipeline SVG strip; published JS bears SRI |
| V12 Files + resources | libvips bound on file size + resolution |
| V13 API + Web service | OpenAPI 3.1 contract |
| V14 Configuration | values.yaml + Helm pin |

## Per-pack overlays

### pack-kr (KR PIPA + KR-FSS + 전자문서법 + ISMS-P)

| Control | Citation | Sites implementation |
|---|---|---|
| Audit-chain integrity | 전자문서법 Art. 5 | Ed25519 + Merkle per Bominal ADR-0028 |
| Special-category data | KR PIPA Art. 23 | data-class `SENSITIVE_PIPA_ART23` on CMS-collection fields; Cedar refusal of anonymous rendering |
| Retention floor | KR-FSS guidelines | 1825d (5y) for financial-sector tenants |
| Notification | KR PIPA Art. 34 | 72h notification per incident-response.md |
| Cross-border | KR PIPA Art. 17 | per-pack residency; cross-pack SCC-gated |
| ISMS-P | KISA Notice 2024-X | annual recertification |

### pack-eu (GDPR + ePrivacy + EU AI Act + EU DSA + eIDAS + NIS2)

| Control | Citation | Sites implementation |
|---|---|---|
| Lawful basis | GDPR Art. 6 | per-purpose admission via Cedar; `legal/ropa.md` |
| Right to erasure | GDPR Art. 17 | page-usecase erasure orchestrator + legal-hold reconciliation |
| Right to portability | GDPR Art. 20 | site-export endpoint |
| DPIA | GDPR Art. 35 | this DPIA |
| Cross-border | Chapter V | per-pack EU residency; SCC for cross-pack |
| AI Act transparency | Art. 50 | T2 AI-page-build labelled "AI is suggesting this page — review before publish"; Art. 14 cancel window 30s |
| AI Act high-risk | Annex III §3 | T2 HR/legal/medical-context overlays REFUSED at Cedar layer pending ADR-SITES-XXXX |
| ePrivacy | Art. 5(3) | analytics first-party only; consent banner required for non-strictly-necessary cookies |
| DSA Art. 14 transparency | EU DSA | publish-refusal records carry policy citation |
| eIDAS Art. 26 AdES | EU 910/2014 | audit-chain Ed25519 satisfies |

### pack-us-healthcare (HIPAA + BAA + ADA Title III + Section 508)

| Control | Citation | Sites implementation |
|---|---|---|
| Security Rule | 45 CFR §164.308 | Risk Analysis + audit controls + encryption |
| Privacy Rule | 45 CFR §164.502(b) | minimum-necessary on CMS-collection fields; PHI data-class |
| Encryption | 45 CFR §164.312(a)(2)(iv) | Tenant-DEK envelope at rest; TLS 1.3 in transit |
| Audit controls | 45 CFR §164.312(b) | Ed25519 + Merkle audit-chain; retention ≥ 6y |
| BAA | 45 CFR §164.504(e) | per-tenant BAA per `legal/baa-template.md`; LEAN refuse pack-us-healthcare without `baa_on_file=true` |
| ADA Title III + Section 508 | (federal) | patient-portal sites refuse publish at < 100% WCAG 2.2 AA |

### pack-us (CCPA / CPRA / sectoral)

| Control | Citation | Sites implementation |
|---|---|---|
| Right to know | CCPA §1798.100 | per-user export |
| Right to delete | CCPA §1798.105 | erasure orchestrator |
| Sale of PD opt-out | CCPA §1798.120 | no sale; `legal/sub-processors.md` |

### pack-jp (APPI)

| Control | Citation | Sites implementation |
|---|---|---|
| Purpose | APPI Art. 17 | tenant onboarding consent |
| Leak notification | APPI Art. 22 | 3-business-day per incident-response.md |
| Cross-border | APPI Art. 24 | pack-jp jp-tokyo-1; cross-pack consent-gated |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays summarised in `policy/data-residency.md`.

Highlights:
- **PDPA 2012 (sg)**: Part III Protection + Part IV Retention + Part VI Transfer.
- **APP 8 + APP 11 + APP 12** (Privacy Act 1988 au).
- **DPDPA 2023 (in)**: §6-16 consent / notice / security.
- **LGPD Arts. 33-36 (br)**: cross-border.
- **UAE PDPL** + **KSA PDPL**: cross-border + impact assessment.

## Audit cadence

| Audit | Cadence | Owner |
|---|---|---|
| SOC 2 Type 2 | annually | external SOC 2 firm |
| ISO 27001:2022 | every 3 years (initial) + annual surveillance | external ISO firm |
| WCAG 2.2 AA conformance review | per-publish (LEAN) + annual external audit | ops-security + external accessibility firm |
| GDPR / EDPB DPA examination | on-tenant request + breach trigger | council-privacy |
| HIPAA OCR audit | on-trigger | external HIPAA firm |
| EU AI Act Annex III §3 conformity (HR/legal/medical overlay) | per-launch + annual | council-privacy + external AI firm |
| EU DSA transparency report | annually | council-privacy |
| Pen-test (boundary tests per threat-model.md) | annually | external pen-test firm |
| LEAN-gate evidence review | per-PR | reviewer-agent |

## References

- ADR-0028, ADR-0117, ADR-0135, ADR-0140, ADR-SITES-0006.
- `threat-model.md`, `dpia.md`, `policy/*`, `multi-region.md`,
  `incident-response.md`, `legal/*`.
- SOC 2 TSC 2017 + 2022 PoF.
- ISO 27001:2022 Annex A.
- GDPR (Regulation 2016/679); EDPB Guidelines 4/2019 + 9/2022.
- KR PIPA + ISMS-P + 전자문서법; PIPC Notice 2020-7.
- HIPAA 45 CFR §164.
- APPI; PDPA; APP; DPDPA; LGPD; UAE PDPL; KSA PDPL.
- EU AI Act Regulation (EU) 2024/1689.
- EU DSA Regulation (EU) 2022/2065.
- ePrivacy Directive 2002/58/EC.
- ADA Title III + Section 508 + WCAG 2.2.
- ATAG 2.0 — w3.org/TR/ATAG20.
- OWASP ASVS v4.
- W3C Subresource Integrity.
- SLSA L3 — slsa.dev.
- NIST SSDF SP 800-218.
- CIS Kubernetes Benchmark.
- eIDAS Regulation (EU) 910/2014.
- NIS2 Directive (EU) 2022/2555.

---



## §day-one-cert-readiness
This anchor is closed for `sites` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `sites` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +13 more.
- Example: `T0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `T0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `sites` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `T0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `sites` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`, `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `T0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.sites.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `sites` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `T0-suggest` touches those data classes.
- Signal sources: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`; +12 more.
- Example event class: `oya.sites.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `sites` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.sites.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `T0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `T0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `sites` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`, `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`; +4 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `sites.block` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `sites` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`; +12 more.
- Example: `T0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.sites` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/sites/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.
- Example: `T0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `sites` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`, `microservices/sites/iac/helm/Chart.yaml`, `microservices/sites/iac/helm/templates/cronjob.yaml`, `microservices/sites/iac/helm/templates/deployment.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `T0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `sites` is in annual full-scope pentest and every major `T0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`, `microservices/sites/iac/helm/Chart.yaml`, `microservices/sites/iac/helm/templates/cronjob.yaml`, `microservices/sites/iac/helm/templates/deployment.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `sites` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `sites` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `T0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `sites` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/sites/catalog/oya-sites-block-adapter-loro.yaml`, `microservices/sites/catalog/oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub.yaml`, `microservices/sites/catalog/oya-sites-cdn-delivery-adapter-libvips.yaml`, `microservices/sites/catalog/oya-sites-cdn-delivery-adapter-pandoc.yaml`, `microservices/sites/catalog/oya-sites-cdn-delivery-adapter-s3.yaml`, `microservices/sites/catalog/oya-sites-cdn-delivery-app.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `T0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `sites` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `T0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `T0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `sites` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `T0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `sites`; owner `axis-sites`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`; +1 more.
- Capability records cited: `microservices/sites/capabilities/T0-suggest.yaml`, `microservices/sites/capabilities/T1-assist.yaml`, `microservices/sites/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar/policy artifacts cited: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/sites/contracts/asyncapi/sites-events.yaml`, `microservices/sites/contracts/openapi/sites.yaml`, `microservices/sites/contracts/proto/sites.proto`.
- Cedar binding: `microservices/sites/policy/auditor-scope.cedar`, `microservices/sites/policy/ci-scope.cedar`, `microservices/sites/policy/data-residency.md`, `microservices/sites/policy/editor-isolation.md`, `microservices/sites/policy/public-read.cedar`, `microservices/sites/policy/tenant-scope.cedar`.
- State/event binding: `sites.block`, `sites.cdn_delivery`, `sites.cms_collection`, `sites.domain_binding`, `sites.page`, `sites.search`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, `microservices/sites/slos/acme-renew-latency.openslo.yaml`, `microservices/sites/slos/cms-query-latency.openslo.yaml`, `microservices/sites/slos/image-optimize-latency.openslo.yaml`, `microservices/sites/slos/page-render-latency.openslo.yaml`, `microservices/sites/slos/publish-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, `microservices/sites/runbooks/ai-page-build-rollback.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, `microservices/sites/runbooks/custom-domain-dns-drift.md`, `microservices/sites/runbooks/page-export-corruption.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `sites`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `sites`.
- `policy-engine` supplies the signed Cedar corpus while `sites` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `sites` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `sites`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `sites` applies the most restrictive policy and emits a degraded-mode audit event.
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
