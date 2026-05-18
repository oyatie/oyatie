---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: calendar
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140]
doc_status: published
---

# Compliance Matrix — calendar µservice

## Purpose

Enumerate compliance frameworks engaged by calendar, the controls satisfied, and where each control is evidenced (per artifact, lane, or runbook).

## Frameworks engaged

### Globally enforced

| Framework | Scope | Mapping |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x–CC9.x | §"SOC 2 Mapping" below |
| ISO 27001:2022 | Annex A.5–A.8 | §"ISO 27001 Mapping" below |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 | §"GDPR Mapping" below |

### Pack-overlays

| Pack | Frameworks |
|---|---|
| pack-kr | KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2/33 + ISMS-P §2.1-2.12 + 전자문서법 Arts. 5/6/7 |
| pack-us-healthcare | HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502/§164.504/§164.512/§164.514 + state-level (CCPA / CMIA / NY SHIELD) |
| pack-eu | GDPR + EDPB Guidelines 4/2019 + 9/2022 + NIS2 + eIDAS 910/2014 |
| pack-jp | APPI Arts. 17/18/20/21/23/24/26-2/27 |
| pack-sg | PDPA 2012 Parts III/IV/VI + MAS Notice 644 |
| pack-au | Privacy Act 1988 APP 1-13 + APRA-CPS 234 |
| pack-in | DPDPA 2023 §6-11 + RBI Master Direction on IT Outsourcing |
| pack-br | LGPD Arts. 6/7/11/14/18/33/38/46/48 + BACEN Res. 4.893/2021 |
| pack-ae | UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15/22/23 |
| pack-ksa | KSA PDPL Royal Decree M/19/2021 Arts. 4-9 + SAMA Cybersecurity Framework 2017 |

### Clinical-scheduling overlays

| Framework | Engaged when | Notes |
|---|---|---|
| FDA 21 CFR Part 11 (electronic records / electronic signatures) | clinical-scheduling tenant in pack-us-healthcare | audit-chain Ed25519 seal satisfies 21 CFR §11.10(e) audit trail + §11.50 electronic signature |
| ICH GCP E6(R2) | clinical-research-scheduling tenant | retention + integrity per §4.9 + §5.5 |

## SOC 2 Mapping

| TSC | Control | Calendar evidence |
|---|---|---|
| CC1.1 | Demonstrates commitment to integrity | council-architecture + ops-security sign-off on threat-model + DPIA |
| CC2.1 | Communicates information | This compliance.md + PRD + runbooks |
| CC3.1 | Specifies risk objectives | threat-model.md + dpia.md |
| CC4.1 | Demonstrates evaluation | audit-chain emission per event + LEAN check coverage |
| CC4.2 | Selects monitoring activities | observability dashboards + per-changeset evidence |
| CC5.1 | Selects + develops control activities | LEAN CI lanes + Cedar policies |
| CC6.1 | Logical access — restricts | per-tenant RLS + tenant-DEK + Cedar policies |
| CC6.2 | Authenticates | OIDC + MFA + per-tenant API keys |
| CC6.3 | Authorises | Cedar policies (`tenant-scope.cedar`, etc.) |
| CC6.6 | Restricts physical access | inherited from cloud-k8s + cloud-secrets |
| CC6.7 | Restricts info flow | type-narrowed projections + LEAN checks |
| CC6.8 | Prevents unauthorized software | branch-protection + signed commits + signed artifacts |
| CC7.1 | Detects security events | observability alerts + audit-chain |
| CC7.2 | Monitors system components | SLO + burn-rate dashboards |
| CC7.4 | Responds to incidents | incident-response.md + runbooks |
| CC7.5 | Recovers from incidents | DR drills + backup retention |
| CC8.1 | Manages changes | ADR-0110 ChangeSet + ADR-0139 SLO-gated promotion |
| CC9.1 | Identifies risks | residual-risk acceptance in threat-model.md |

## ISO 27001 Mapping

| Annex A Control | Calendar evidence |
|---|---|
| A.5.7 (threat intelligence) | threat-model.md re-review triggers |
| A.5.10 (acceptable use of info) | policy/event-isolation.md |
| A.5.14 (information transfer) | policy/data-residency.md |
| A.5.15 (access control) | Cedar policies + RLS |
| A.5.17 (authentication info) | OpenBao + per-tenant DEK rotation |
| A.5.23 (cloud service usage) | ADR-0117 |
| A.5.26 (response to security incidents) | incident-response.md |
| A.5.27 (lessons from incidents) | post-incident review process |
| A.5.28 (collection of evidence) | audit-chain seal |
| A.5.30 (ICT continuity) | multi-region.md |
| A.5.31 (legal + statutory) | This compliance.md per-pack overlays |
| A.5.32 (intellectual property) | dependencies + licenses in `catalog/*.yaml` |
| A.5.33 (records protection) | retention + legal-hold |
| A.5.34 (privacy + PII) | dpia.md + policy/* |
| A.8.2 (privileged access rights) | OpenBao JIT + 2-person rule |
| A.8.3 (info access restriction) | RLS + Cedar |
| A.8.5 (secure authentication) | OIDC + MFA |
| A.8.7 (protection against malware) | inherited from cloud-k8s |
| A.8.11 (data masking) | redaction in cross-tenant projection + .ics export |
| A.8.12 (data leakage prevention) | LEAN checks + DLP scan |
| A.8.15 (logging) | observability + audit-chain |
| A.8.16 (monitoring activities) | dashboards + alerts |
| A.8.20 (network security) | mesh mTLS + NetworkPolicy |
| A.8.21 (security of network services) | per-tenant API key + rate limits |
| A.8.23 (web filtering) | WAF at ingress |
| A.8.25 (secure development lifecycle) | LEAN gates + ADR-0139 SLO-gated promotion |
| A.8.26 (application security requirements) | per-microservice security artifacts |
| A.8.27 (secure system architecture) | ADR-0056 + ADR-0105 |
| A.8.28 (secure coding) | LEAN check `oya-check-ics-parser-conformance` + cargo fuzz |
| A.8.32 (change management) | ADR-0110 + branch-protection |
| A.8.33 (test information) | synthetic test tenants per `ci-scope.cedar` |
| A.8.34 (audit + protection of audit systems) | audit-chain immutability + 2-person rule on admin |

## GDPR Mapping

| Article | Calendar evidence |
|---|---|
| Art. 5(1)(a) lawfulness | per-purpose lawful basis in dpia.md |
| Art. 5(1)(b) purpose limitation | dpia.md §2.4 |
| Art. 5(1)(c) data minimisation | type-narrowed cross-tenant projection + redaction |
| Art. 5(1)(d) accuracy | tenant-edit UX + audit history |
| Art. 5(1)(e) storage limitation | retention per pack |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK + Ed25519 audit-chain |
| Art. 6(1) lawful basis | dpia.md §2.4 |
| Art. 9 special-category | pack-us-healthcare overlay + pack-kr flagged-event |
| Art. 13/14 transparency | tenant DPA template |
| Art. 17 right-to-erasure | DSR cascade + hold-vs-erasure policy |
| Art. 22 automated decision | scheduling is operational, not legal-effect on subject |
| Art. 25 by design + default | type-system separation + Cedar policy |
| Art. 28 processor agreement | DPA template |
| Art. 30 records of processing | RoPA template |
| Art. 32 security of processing | every STRIDE / LINDDUN mitigation |
| Art. 33 breach notification | incident-response.md 72h chain |
| Art. 35 DPIA | dpia.md |
| Arts. 44-50 transfers | SCC + transfer register + multi-region.md |

## Pack-overlay detail: pack-kr (KR PIPA + ISMS-P + 전자문서법)

| PIPA Article | Calendar evidence |
|---|---|
| Art. 15 (consent for collection) | tenant onboarding consent flow |
| Art. 17 (cross-border transfer) | default-residency + SCC clause |
| Art. 18 (use beyond stated purpose) | dpia.md purpose-limitation §2.4 |
| Art. 22-2 (sensitive personal info, identifier-based) | flagged-event Cedar policy |
| Art. 23 (sensitive personal info) | per-event sensitivity flag + access restrictions |
| Art. 23-2 (cross-border sensitive) | pack-pinning + SCC |
| Art. 24 (uniquely identifying) | hashed tenant ID + salt rotation |
| Art. 25 (CCTV) | N/A (calendar doesn't collect CCTV) |
| Art. 28 (storage period) | retention bounded per asset table |
| Art. 29 (technical safeguards) | 12-safeguard mapping in threat-model.md |
| Art. 29-2 (data leakage prevention) | LEAN checks + DLP |
| Art. 33 (DPIA / 영향평가) | dpia.md |

| ISMS-P §§ | Calendar evidence |
|---|---|
| §2.1 (information security policy) | this compliance.md + policy/* |
| §2.3 (asset management) | catalog/*.yaml |
| §2.5 (human security) | 2-person rule + JIT |
| §2.7 (access control) | RLS + Cedar |
| §2.9 (operational security) | runbooks/* |
| §2.10 (communications security) | mesh mTLS + WAF |
| §2.11 (cryptography) | tenant-DEK + audit-chain Ed25519 |
| §2.12 (incident management) | incident-response.md |

| 전자문서법 §§ | Evidence |
|---|---|
| Art. 5 (integrity of electronic documents) | audit-chain Ed25519 |
| Art. 6 (storage of electronic documents) | retention + legal hold |
| Art. 7 (e-signature equivalence) | OIDC + JIT |

## Pack-overlay detail: pack-us-healthcare (HIPAA)

| 45 CFR §§ | Calendar evidence |
|---|---|
| §164.308(a)(1)(ii)(A) risk analysis | dpia.md + threat-model.md |
| §164.308(a)(3) workforce security | OpenBao JIT + 2-person rule |
| §164.308(a)(4) info access management | Cedar + RLS |
| §164.310(a) facility access | inherited from cloud-k8s |
| §164.312(a) access control | RLS + Cedar |
| §164.312(b) audit controls | audit-chain Ed25519 + retention ≥ 6y |
| §164.312(c) integrity | audit-chain Merkle |
| §164.312(d) person authentication | OIDC + MFA |
| §164.312(e) transmission security | mesh mTLS |
| §164.314(a) BAA | legal/baa-template.md |
| §164.316 documentation | retain artifacts ≥ 6y |
| §164.502(a) Permitted Uses (TPO) | tenant DPA |
| §164.502(b) Minimum Necessary | cross-tenant projection type-narrowing |
| §164.504(e) BAA terms | BAA template |
| §164.512 disclosures permitted | dpia.md |
| §164.514 de-identification | redaction in .ics export |

State-level:
- CCPA Cal. Civ. Code §1798.100 et seq.: GDPR-Art-15 equivalent, DSR cascade satisfies.
- CMIA Cal. Civ. Code §56 et seq.: medical info disclosure restrictions; pack-us-healthcare enforces.
- NY SHIELD Act: breach notification + reasonable security; integrated.

## Pack-overlay detail: pack-eu (GDPR + EDPB + NIS2 + eIDAS)

- **EDPB Guidelines 4/2019 (Art. 25)**: by-design + by-default verified in §4 of dpia.md.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h chain in incident-response.md.
- **NIS2 (2022/2555)**: 24h + 72h + 1mo timelines when threshold-engaged.
- **eIDAS 910/2014 Art. 26**: audit-chain Ed25519 seal satisfies AdES.
- **Schrems II + Arts. 44-46**: SCC-only transfers + TIA when non-adequate.

## Pack-overlay detail: pack-jp (APPI)

| APPI Article | Evidence |
|---|---|
| Art. 17 (purpose) | tenant DPA |
| Art. 18 (acquisition by deception) | N/A |
| Art. 20 (security control) | every STRIDE mitigation |
| Art. 21 (cross-border) | pack-pinning |
| Art. 23 (joint use) | tenant DPA |
| Art. 24 (third-party provision) | sub-processor list |
| Art. 26-2 (cross-border consent) | tenant DPA |
| Art. 27 (consent for sensitive) | flagged-event consent |

## Pack-overlay detail: pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/calendar-compliance-overlay.md`.

Highlights:
- **PDPA 2012**: Part III Protection Obligation + Part IV Retention Limitation + Part VI Transfer Limitation.
- **APP 8 + APP 11 + APP 12** (Privacy Act 1988): cross-border + security + access.
- **APRA-CPS 234**: information security for financial-services tenants.
- **DPDPA 2023**: §6-11 consent/notice/security.
- **LGPD Arts. 33-36**: cross-border transfer.
- **UAE PDPL** + **KSA PDPL**: cross-border + impact assessment.

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
| LEAN-gate evidence review | per-PR | reviewer-agent |

## References

- ADR-0028 (Bominal), ADR-0117, ADR-0135, ADR-0140.
- `threat-model.md`, `dpia.md`, `policy/*`, `multi-region.md`, `incident-response.md`, `legal/*`.
- SOC 2 TSC 2017 + 2022 PoF.
- ISO 27001:2022 Annex A.
- GDPR (Regulation 2016/679); EDPB Guidelines 4/2019 + 9/2022.
- KR PIPA + ISMS-P + 전자문서법; PIPC Notice 2020-7.
- HIPAA 45 CFR §164; FDA 21 CFR Part 11.
- APPI; PDPA; APP; DPDPA; LGPD; UAE PDPL; KSA PDPL.

## Per-Pack Compliance Overlay Sections (2026-05-17 additive)

These overlays append per ADR-0133 11-pack-overlay program with the
concrete compliance-control delta for each pack. Each overlay maps
to named statutory / regulatory citations.

### pack-kr (KR PIPA + KR-FSS + 전자문서법 + ISMS-P)

| Control | Citation | Calendar implementation |
|---|---|---|
| Audit-chain integrity | 전자문서법 Art. 5 | Ed25519 + Merkle per Bominal ADR-0028; tamper detection on read |
| Special-category data | KR PIPA Art. 23 | data-class `SENSITIVE_PIPA_ART23` on relationship-graph fields; Cedar refusal |
| Retention floor | KR-FSS guidelines | 1825d (5y) for financial-sector tenants; enforced at event-store-domain |
| Notification | KR PIPA Art. 34 | 72h notification per incident-response.md |
| Cross-border | KR PIPA Art. 17 | per-pack residency; cross-pack SCC-gated |
| ISMS-P | KISA Notice 2024-X | annual ISMS-P recertification; CalDAV backend (Radicale 3.2.3 LTS) within scope |

### pack-eu (GDPR + ePrivacy + EU AI Act)

| Control | Citation | Calendar implementation |
|---|---|---|
| Lawful basis | GDPR Art. 6 | per-purpose admission via Cedar; per `legal/ropa.md` records of processing |
| Right to erasure | GDPR Art. 17 | event-store-usecase erasure orchestrator + legal-hold reconciliation |
| Right to portability | GDPR Art. 20 | .ics export per PRD FR-08 |
| DPIA | GDPR Art. 35 | this DPIA |
| Cross-border | Chapter V | per-pack EU residency; SCC for cross-pack |
| AI Act high-risk | Annex III §3 | T1/T2 HR-context overlays REFUSED at Cedar layer pending ADR-CAL-XXXX conformity assessment |
| ePrivacy | Art. 5(3) | web-UI tracking-free posture |

### pack-us (CCPA / CPRA / sectoral)

| Control | Citation | Calendar implementation |
|---|---|---|
| Right to know | CCPA §1798.100 | per-user export per PRD FR-08 |
| Right to delete | CCPA §1798.105 | event-store deletion orchestrator |
| Sale of PD opt-out | CCPA §1798.120 | no sale; documented in `legal/sub-processors.md` |
| SOC 2 | TSC 2017+2022 | annual SOC 2 Type 2 |

### pack-us-healthcare (HIPAA + BAA + FDA 21 CFR Part 11)

| Control | Citation | Calendar implementation |
|---|---|---|
| Security Rule | 45 CFR §164.308 | Risk Analysis + audit controls + encryption |
| Privacy Rule | 45 CFR §164.502(b) | minimum-necessary: data-class PHI on appointment fields |
| Encryption | 45 CFR §164.312(a)(2)(iv) | Tenant-DEK envelope at rest; TLS 1.3 in transit |
| Audit controls | 45 CFR §164.312(b) | Ed25519 + Merkle audit-chain |
| BAA | 45 CFR §164.504(e) | per-tenant BAA per `legal/baa-template.md` |
| FDA Part 11 | 21 CFR §11.10 | electronic records integrity for HIPAA-covered tenants |
| CalDAV backend | (operational) | SabreDAV 4.6 per ADR-CAL-0001 for healthcare-specific scheduling workflows |

### pack-jp (APPI)

| Control | Citation | Calendar implementation |
|---|---|---|
| Specified-purpose | APPI Art. 17 | consent-recorded purposes per tenant onboarding |
| Leak notification | APPI Art. 22 | 3-business-day notification per incident-response.md |
| Cross-border | APPI Art. 24 | per-pack jp-tokyo-1; cross-pack consent-gated |

### pack-sg (PDPA Singapore)

| Control | Citation | Calendar implementation |
|---|---|---|
| Consent | PDPA §13 | Cedar admission + recorded consent |
| Protection | PDPA §24 | encryption + audit-chain |
| Cross-border | PDPA §26 | comparable-protection assessment for cross-pack |

### pack-au (Privacy Act 1988)

| Control | Citation | Calendar implementation |
|---|---|---|
| Collection limitation | APP 3 | data-class enforcement on event fields |
| Cross-border | APP 8 | per-pack au-sydney-1; cross-pack OAIC-accountable |
| Security | APP 11 | TLS + encryption + audit-chain |

### pack-in (DPDPA 2023)

| Control | Citation | Calendar implementation |
|---|---|---|
| Notice + consent | DPDPA §6 | tenant-onboarding consent flow |
| Significant data fiduciary | DPDPA §10 | DPO + audit per §10(2) for healthcare tenants |
| Cross-border | DPDPA §16 | whitelist-based |

### pack-br (LGPD)

| Control | Citation | Calendar implementation |
|---|---|---|
| Lawful basis | LGPD Art. 7 | Cedar admission per recorded basis |
| Cross-border | LGPD Art. 33 | ANPD-approved mechanism for cross-pack |
| Reports of processing | LGPD Art. 37 | per `legal/ropa.md` |

### pack-ae (UAE PDPL + Federal Decree 45/2021)

| Control | Citation | Calendar implementation |
|---|---|---|
| Consent | PDPL Art. 5 | tenant-onboarding consent flow |
| Cross-border | PDPL Art. 22 | UAE DPA-approved mechanism for cross-pack |
| Security | PDPL Art. 20 | encryption + audit-chain |
| Hijri overlay | (operational) | ICU4X `icu_calendar` per ADR-CAL-0004 |

### pack-ksa (KSA PDPL + Royal Decree M/19)

| Control | Citation | Calendar implementation |
|---|---|---|
| Lawful processing | PDPL Art. 6 | Cedar admission per recorded basis |
| Cross-border | PDPL Art. 29 | SDAIA-approved mechanism for cross-pack |
| Sharia retention | (operational; per Sharia-court rulings) | per-tenant retention extension supported |
| Hijri overlay | (operational) | ICU4X `icu_calendar` per ADR-CAL-0004 |
