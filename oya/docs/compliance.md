---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: docs
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140 (retired per ADR-0145), ADR-DOCS-0001, ADR-DOCS-0003, ADR-DOCS-0004, ADR-DOCS-0005, ADR-DOCS-0006]
doc_status: published
---

# Compliance Matrix — docs µservice

## Purpose

Enumerate compliance frameworks engaged by docs, the controls satisfied, and where each control is evidenced.

## Frameworks engaged

### Globally enforced

| Framework | Scope | Mapping |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x–CC9.x | §"SOC 2 Mapping" below |
| ISO 27001:2022 | Annex A.5–A.8 | §"ISO 27001 Mapping" below |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 | §"GDPR Mapping" below |
| WCAG 2.2 AA | Accessibility surface (editor + export) | §"WCAG 2.2 AA" below |
| SLSA Level 3 | Supply-chain (CRDT library, Pandoc, WeasyPrint, Chromium, ClamAV) | §"SLSA L3" below |
| NIST SSDF | Secure development | §"NIST SSDF" below |
| OWASP ASVS v4.0 | Web service | §"OWASP ASVS" below |
| CIS Kubernetes Benchmark | Cluster substrate | §"CIS K8s" below |

### Pack-overlays

| Pack | Frameworks |
|---|---|
| pack-kr | KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2/33 + ISMS-P §2.1-2.12 + 전자문서법 Arts. 5/6/7 |
| pack-us-healthcare | HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502/§164.504/§164.512/§164.514 + state-level (CCPA / CMIA / NY SHIELD) |
| pack-eu | GDPR + EDPB Guidelines 4/2019 + 9/2022 + NIS2 + eIDAS 910/2014 (PAdES) + EU AI Act 2024/1689 |
| pack-jp | APPI Arts. 17/18/20/21/23/24/26-2/27 |
| pack-sg | PDPA 2012 Parts III/IV/VI + MAS Notice 644 |
| pack-au | Privacy Act 1988 APP 1-13 + APRA-CPS 234 |
| pack-in | DPDPA 2023 §6-11 + RBI Master Direction on IT Outsourcing |
| pack-br | LGPD Arts. 6/7/11/14/18/33/38/46/48 + BACEN Res. 4.893/2021 |
| pack-ae | UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15/22/23 |
| pack-ksa | KSA PDPL Royal Decree M/19/2021 Arts. 4-9 + SAMA Cybersecurity Framework 2017 |

### Clinical-content overlays

| Framework | Engaged when | Notes |
|---|---|---|
| FDA 21 CFR Part 11 | clinical-notes tenant in pack-us-healthcare | audit-chain Ed25519 seal satisfies §11.10(e) audit trail + §11.50 electronic signature |
| ICH GCP E6(R2) | clinical-research-document tenant | retention + integrity per §4.9 + §5.5 |

## SOC 2 Mapping

| TSC | Control | Docs evidence |
|---|---|---|
| CC1.1 | Demonstrates commitment to integrity | council-architecture + ops-security sign-off on threat-model + DPIA |
| CC2.1 | Communicates information | this compliance.md + PRD + runbooks |
| CC3.1 | Specifies risk objectives | threat-model.md + dpia.md |
| CC4.1 | Demonstrates evaluation | audit-chain emission per doc lifecycle + LEAN check coverage |
| CC4.2 | Selects monitoring activities | observability dashboards + per-changeset evidence |
| CC5.1 | Selects + develops control activities | LEAN CI lanes + Cedar policies |
| CC6.1 | Logical access — restricts | per-tenant RLS + tenant-DEK + Cedar policies + per-block ACL |
| CC6.2 | Authenticates | OIDC + MFA + per-tenant API keys + share-link Ed25519 |
| CC6.3 | Authorises | Cedar policies |
| CC6.6 | Restricts physical access | inherited from cloud-k8s + cloud-secrets |
| CC6.7 | Restricts info flow | embed-resolver source-side ACL passthrough + per-block ACL |
| CC6.8 | Prevents unauthorized software | branch-protection + signed commits + signed artifacts |
| CC7.1 | Detects security events | observability alerts + audit-chain |
| CC7.2 | Monitors system components | SLO + burn-rate dashboards |
| CC7.4 | Responds to incidents | incident-response.md + runbooks |
| CC7.5 | Recovers from incidents | DR drills + backup retention |
| CC8.1 | Manages changes | ADR-0110 ChangeSet + ADR-0139 SLO-gated promotion |
| CC9.1 | Identifies risks | residual-risk acceptance in threat-model.md |

## ISO 27001 Mapping

| Annex A Control | Docs evidence |
|---|---|
| A.5.7 (threat intelligence) | threat-model.md re-review triggers |
| A.5.10 (acceptable use) | policy/editor-isolation.md |
| A.5.14 (information transfer) | policy/data-residency.md |
| A.5.15 (access control) | Cedar + RLS + per-block ACL |
| A.5.17 (authentication info) | OpenBao + per-tenant DEK rotation + share-link key rotation |
| A.5.23 (cloud service usage) | ADR-0117 |
| A.5.26 (response to security incidents) | incident-response.md |
| A.5.27 (lessons from incidents) | post-incident review process |
| A.5.28 (collection of evidence) | audit-chain seal |
| A.5.30 (ICT continuity) | multi-region.md |
| A.5.31 (legal + statutory) | this compliance.md per-pack overlays |
| A.5.32 (intellectual property) | dependencies + licenses in `catalog/*.yaml` |
| A.5.33 (records protection) | retention + legal-hold (Object Lock) |
| A.5.34 (privacy + PII) | dpia.md + policy/* |
| A.8.2 (privileged access rights) | OpenBao JIT + 2-person rule |
| A.8.3 (info access restriction) | RLS + Cedar + per-block ACL |
| A.8.5 (secure authentication) | OIDC + MFA |
| A.8.7 (protection against malware) | ClamAV / OPSWAT attachment scan; HTML sanitiser |
| A.8.11 (data masking) | redaction in cross-tenant projection + export |
| A.8.12 (data leakage prevention) | LEAN checks + DLP scan + gVisor egress block |
| A.8.15 (logging) | observability + audit-chain |
| A.8.16 (monitoring activities) | dashboards + alerts |
| A.8.20 (network security) | mesh mTLS + NetworkPolicy |
| A.8.21 (security of network services) | per-tenant API key + rate limits |
| A.8.23 (web filtering) | WAF at ingress |
| A.8.25 (secure development lifecycle) | LEAN gates + ADR-0139 SLO-gated promotion |
| A.8.26 (application security requirements) | per-microservice security artifacts |
| A.8.27 (secure system architecture) | ADR-0056 + ADR-0105 |
| A.8.28 (secure coding) | LEAN check `oya-check-ooxml-import-fidelity` + cargo fuzz |
| A.8.32 (change management) | ADR-0110 + branch-protection |
| A.8.33 (test information) | synthetic test tenants per `ci-scope.cedar` |
| A.8.34 (audit + protection of audit systems) | audit-chain immutability + 2-person rule on admin |

## GDPR Mapping

| Article | Docs evidence |
|---|---|
| Art. 5(1)(a) lawfulness | per-purpose lawful basis in dpia.md |
| Art. 5(1)(b) purpose limitation | dpia.md §2.4 |
| Art. 5(1)(c) data minimisation | per-block ACL + redaction in export |
| Art. 5(1)(d) accuracy | tenant-edit UX + audit history |
| Art. 5(1)(e) storage limitation | retention per pack |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK + Ed25519 audit-chain |
| Art. 6(1) lawful basis | dpia.md §2.4 |
| Art. 9 special-category | pack-us-healthcare overlay + pack-kr flagged-doc |
| Art. 13/14 transparency | tenant DPA template + AI-assist labelling |
| Art. 17 right-to-erasure | DSR cascade + hold-vs-erasure policy |
| Art. 22 automated decision | AI-assist is suggestion-only; no legal-effect on subject (unless pack-eu HR T1/T2 which is refused at Cedar) |
| Art. 25 by design + default | type-system separation + Cedar policy + per-block ACL |
| Art. 28 processor agreement | DPA template |
| Art. 30 records of processing | RoPA template |
| Art. 32 security of processing | every STRIDE / LINDDUN mitigation |
| Art. 33 breach notification | incident-response.md 72h chain |
| Art. 35 DPIA | dpia.md |
| Arts. 44-50 transfers | SCC + transfer register + multi-region.md |

## Pack-overlay detail: pack-kr (KR PIPA + ISMS-P + 전자문서법)

| PIPA Article | Docs evidence |
|---|---|
| Art. 15 (consent for collection) | tenant onboarding consent flow |
| Art. 17 (cross-border transfer) | default-residency + SCC clause |
| Art. 18 (use beyond stated purpose) | dpia.md purpose-limitation §2.4 |
| Art. 22-2 (sensitive personal info, identifier-based) | flagged-doc Cedar policy |
| Art. 23 (sensitive personal info) | per-doc sensitivity flag + access restrictions |
| Art. 23-2 (cross-border sensitive) | pack-pinning + SCC |
| Art. 24 (uniquely identifying) | hashed tenant ID + salt rotation |
| Art. 28 (storage period) | retention bounded per asset table |
| Art. 29 (technical safeguards) | 12-safeguard mapping in threat-model.md |
| Art. 29-2 (data leakage prevention) | LEAN checks + DLP |
| Art. 33 (DPIA / 영향평가) | dpia.md |

| ISMS-P §§ | Docs evidence |
|---|---|
| §2.1 (information security policy) | this compliance.md + policy/* |
| §2.3 (asset management) | catalog/*.yaml |
| §2.5 (human security) | 2-person rule + JIT |
| §2.7 (access control) | RLS + Cedar + per-block ACL |
| §2.9 (operational security) | runbooks/* |
| §2.10 (communications security) | mesh mTLS + WAF |
| §2.11 (cryptography) | tenant-DEK + audit-chain Ed25519 + share-link signing |
| §2.12 (incident management) | incident-response.md |

| 전자문서법 §§ | Evidence |
|---|---|
| Art. 5 (integrity of electronic documents) | audit-chain Ed25519 |
| Art. 6 (storage of electronic documents) | retention + legal hold (S3 Object Lock) |
| Art. 7 (e-signature equivalence) | OIDC + JIT |

## Pack-overlay detail: pack-us-healthcare (HIPAA)

| 45 CFR §§ | Docs evidence |
|---|---|
| §164.308(a)(1)(ii)(A) risk analysis | dpia.md + threat-model.md |
| §164.308(a)(3) workforce security | OpenBao JIT + 2-person rule |
| §164.308(a)(4) info access management | Cedar + RLS + per-block ACL |
| §164.310(a) facility access | inherited from cloud-k8s |
| §164.312(a) access control | RLS + Cedar + per-block ACL |
| §164.312(b) audit controls | audit-chain Ed25519 + retention ≥ 6y |
| §164.312(c) integrity | audit-chain Merkle |
| §164.312(d) person authentication | OIDC + MFA |
| §164.312(e) transmission security | mesh mTLS |
| §164.314(a) BAA | legal/baa-template.md |
| §164.316 documentation | retain artifacts ≥ 6y |
| §164.502(a) Permitted Uses (TPO) | tenant DPA |
| §164.502(b) Minimum Necessary | per-block ACL type-narrowing |
| §164.504(e) BAA terms | BAA template |
| §164.512 disclosures permitted | dpia.md |
| §164.514 de-identification | redaction in export |

State-level:
- CCPA Cal. Civ. Code §1798.100: GDPR-Art-15 equivalent, DSR cascade satisfies.
- CMIA Cal. Civ. Code §56: medical info disclosure; pack-us-healthcare enforces.
- NY SHIELD Act: breach notification + reasonable security.

## Pack-overlay detail: pack-eu (GDPR + EDPB + NIS2 + eIDAS + EU AI Act)

| Control | Citation | Docs implementation |
|---|---|---|
| Lawful basis | GDPR Art. 6 | per-purpose admission via Cedar |
| Right to erasure | GDPR Art. 17 | document-store-usecase erasure orchestrator + legal-hold reconciliation |
| Right to portability | GDPR Art. 20 | DOCX / Markdown / HTML / PDF / EPUB export per PRD FR-09 |
| DPIA | GDPR Art. 35 | this DPIA |
| Cross-border | Chapter V | per-pack EU residency; SCC for cross-pack |
| AI Act limited-risk (Art. 50) | EU AI Act 2024/1689 | AI-assist labels in UI |
| AI Act high-risk (Annex III §3) | EU AI Act 2024/1689 | T1/T2 HR-context overlays REFUSED at Cedar layer pending ADR-DOCS-0005 conformity assessment |
| ePrivacy | Art. 5(3) | doc web-UI tracking-free posture |
| eIDAS PAdES | 910/2014 | PAdES B-LT signed PDF export per pack-eu overlay |
| NIS2 | 2022/2555 | 24h+72h+1mo incident timelines |

## Pack-overlay detail: pack-us (CCPA / CPRA / sectoral)

| Control | Citation | Docs implementation |
|---|---|---|
| Right to know | CCPA §1798.100 | per-user export per PRD FR-09 |
| Right to delete | CCPA §1798.105 | document-store deletion orchestrator |
| Sale of PD opt-out | CCPA §1798.120 | no sale; documented in `legal/sub-processors.md` |
| SOC 2 | TSC 2017+2022 | annual SOC 2 Type 2 |

## Pack-overlay detail: pack-jp (APPI)

| Control | Citation | Docs implementation |
|---|---|---|
| Specified-purpose | APPI Art. 17 | consent-recorded purposes per tenant onboarding |
| Leak notification | APPI Art. 22 | 3-business-day notification per incident-response.md |
| Cross-border | APPI Art. 24 | per-pack jp-tokyo-1; cross-pack consent-gated |

## Pack-overlay detail: pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/docs-compliance-overlay.md`.

Highlights:
- **PDPA 2012**: Part III Protection + Part IV Retention + Part VI Transfer.
- **APP 8 + APP 11 + APP 12**: cross-border + security + access.
- **APRA-CPS 234**: information security for financial-services tenants.
- **DPDPA 2023**: §6-11.
- **LGPD Arts. 33-36**: cross-border.
- **UAE PDPL** + **KSA PDPL**: cross-border + impact assessment + Sharia retention.

## WCAG 2.2 AA

| Success Criterion | Docs evidence |
|---|---|
| 1.1.1 Non-text content (alt-text required) | block-types kernel enforces alt-text on image blocks; export validation |
| 1.3.1 Info + relationships (heading hierarchy) | block-types schema validates heading order |
| 1.4.3 Contrast (minimum) | design-system primitives enforce; per-export Pa11y check |
| 2.1.1 Keyboard | editor + reader fully keyboard-navigable |
| 2.4.6 Headings + labels | block-types schema enforces |
| 3.3.1 Error identification | suggestion + comment surfaces accessible error states |
| 4.1.2 Name, role, value | ARIA roles on every block primitive |
| 4.1.3 Status messages | live-region announcements for collab updates |

Verification: `oya-governance-wcag-22-aa-conformance` LEAN lane runs axe-core + Pa11y on every export sample + UI test corpus.

## SLSA L3

- All Layer-A dependencies (Loro, Pandoc, WeasyPrint, Chromium, ClamAV, KaTeX) pinned + LTS per ADR-DOCS-0001 / ADR-DOCS-0003.
- All Layer-B crates built reproducibly + signed (sigstore/cosign).
- Provenance attestation per release.

## NIST SSDF

- Per-PR LEAN gates (linting, type-checking, port-location, layer-correctness).
- Per-release threat-model + DPIA review.
- Per-incident post-incident review with corrective actions tracked.

## OWASP ASVS v4.0

- V1: Architecture, design + threat modelling → threat-model.md + ADRs.
- V2: Authentication → OIDC + MFA + share-link Ed25519.
- V3: Session management → WS gateway tenant lease.
- V4: Access control → Cedar + RLS + per-block ACL.
- V5: Validation, sanitisation, encoding → `ammonia` HTML sanitiser + OOXML strict parser.
- V7: Error handling + logging → audit-chain seals.
- V8: Data protection → tenant-DEK envelope + Object Lock.
- V9: Communication → mTLS + TLS 1.3.
- V10: Malicious code → ClamAV + OPSWAT attachment scan.
- V11: Business logic → CRDT op signature + suggestion state machine.
- V12: Files + resources → attachment scanner + size limits.
- V14: Configuration → Helm charts + LTS pins.

## CIS Kubernetes Benchmark

- Inherited from cloud-k8s µservice; docs deployments comply with security-context (runAsNonRoot, readOnlyRootFilesystem, drop ALL caps) per the Helm template.

## Audit cadence

| Audit | Cadence | Owner |
|---|---|---|
| SOC 2 Type 2 | annually | external SOC 2 firm |
| ISO 27001:2022 | every 3 years (initial) + annual surveillance | external ISO firm |
| GDPR / EDPB DPA examination | on-tenant request + breach trigger | council-privacy |
| HIPAA OCR audit | on-trigger | external HIPAA firm |
| PIPC examination | on-trigger | council-privacy |
| ANPD (Brazil) | on-trigger | council-privacy |
| Pen-test (boundary tests per threat-model.md) | annually | external pen-test firm |
| WCAG 2.2 AA audit | bi-annually | external accessibility firm |
| LEAN-gate evidence review | per-PR | reviewer-agent |
| AI Act conformity (HR-context) | per tenant opt-in | council-privacy + axis-docs |

## References

- ADR-0028 (Bominal), ADR-0117, ADR-0135, ADR-0140.
- ADR-DOCS-0001 through ADR-DOCS-0006.
- `threat-model.md`, `dpia.md`, `policy/*`, `multi-region.md`, `incident-response.md`, `legal/*`.
- SOC 2 TSC 2017 + 2022 PoF.
- ISO 27001:2022 Annex A.
- GDPR (Regulation 2016/679); EDPB Guidelines 4/2019 + 9/2022.
- KR PIPA + ISMS-P + 전자문서법; PIPC Notice 2020-7.
- HIPAA 45 CFR §164; FDA 21 CFR Part 11.
- APPI; PDPA; APP; DPDPA; LGPD; UAE PDPL; KSA PDPL.
- WCAG 2.2 (W3C, 2023-10).
- SLSA Specification v1.0.
- NIST SSDF SP 800-218.
- OWASP ASVS v4.0.
- CIS Kubernetes Benchmark v1.9.
- EU AI Act Regulation (EU) 2024/1689; eIDAS Regulation 910/2014.

---



## §day-one-cert-readiness
This anchor is closed for `docs` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `docs` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +13 more.
- Example: `T0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `T0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `docs` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `T0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `docs` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`, `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `T0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.docs.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `docs` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `T0-suggest` touches those data classes.
- Signal sources: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`; +12 more.
- Example event class: `oya.docs.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `docs` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.docs.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `T0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `T0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `docs` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`, `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`; +5 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `docs.block_types` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `docs` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`; +12 more.
- Example: `T0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.docs` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/docs/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.
- Example: `T0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `docs` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`, `microservices/docs/iac/helm/Chart.yaml`, `microservices/docs/iac/helm/templates/deployment.yaml`, `microservices/docs/iac/helm/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `T0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `docs` is in annual full-scope pentest and every major `T0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`, `microservices/docs/iac/helm/Chart.yaml`, `microservices/docs/iac/helm/templates/deployment.yaml`, `microservices/docs/iac/helm/templates/hpa.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `docs` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `docs` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `T0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `docs` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/docs/catalog/oya-docs-block-types-kernel.yaml`, `microservices/docs/catalog/oya-docs-collab-crdt-adapter-valkey.yaml`, `microservices/docs/catalog/oya-docs-collab-crdt-adapter.yaml`, `microservices/docs/catalog/oya-docs-collab-crdt-kernel.yaml`, `microservices/docs/catalog/oya-docs-comments-and-suggestions-adapter-postgres.yaml`, `microservices/docs/catalog/oya-docs-comments-and-suggestions-kernel.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `T0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `docs` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `T0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `T0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `docs` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `T0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `docs`; owner `axis-docs`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `block-types`, `collab-crdt`, `comments-and-suggestions`, `document-store`, `embed-resolver`, `export-import`; +2 more.
- Capability records cited: `microservices/docs/capabilities/T0-suggest.yaml`, `microservices/docs/capabilities/T1-assist.yaml`, `microservices/docs/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar/policy artifacts cited: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/proto/docs.proto`.
- Cedar binding: `microservices/docs/policy/auditor-scope.cedar`, `microservices/docs/policy/ci-scope.cedar`, `microservices/docs/policy/data-residency.md`, `microservices/docs/policy/editor-isolation.md`, `microservices/docs/policy/public-read.cedar`, `microservices/docs/policy/tenant-scope.cedar`.
- State/event binding: `docs.block_types`, `docs.collab_crdt`, `docs.comments_and_suggestions`, `docs.document_store`, `docs.embed_resolver`, `docs.export_import`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/docs/slos/doc-list-latency.openslo.yaml`, `microservices/docs/slos/doc-open-latency.openslo.yaml`, `microservices/docs/slos/export-pdf-latency.openslo.yaml`, `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml`; +3 more.
- Runbook binding: `microservices/docs/runbooks/attachment-restore.md`, `microservices/docs/runbooks/collab-conflict-resolution.md`, `microservices/docs/runbooks/doc-version-restore-corruption.md`, `microservices/docs/runbooks/editor-session-storm-throttle.md`, `microservices/docs/runbooks/embed-source-stale-detection.md`, `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `docs`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `docs`.
- `policy-engine` supplies the signed Cedar corpus while `docs` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `docs` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `docs`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `docs` applies the most restrictive policy and emits a degraded-mode audit event.
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

