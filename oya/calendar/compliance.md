---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: calendar
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140 (retired per ADR-0145)]
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

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/calendar-compliance-overlay.md`.

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

---



## §day-one-cert-readiness
This anchor is closed for `calendar` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `calendar` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +18 more.
- Example: `T0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `T0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `calendar` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `T0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `calendar` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`, `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`; +21 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `T0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.calendar.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `calendar` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `T0-suggest` touches those data classes.
- Signal sources: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`; +12 more.
- Example event class: `oya.calendar.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `calendar` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.calendar.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `T0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `T0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `calendar` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`, `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`; +3 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `calendar.availability_resolver` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `calendar` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`; +12 more.
- Example: `T0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.calendar` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/calendar/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.
- Example: `T0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `calendar` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`, `microservices/calendar/iac/helm/Chart.yaml`, `microservices/calendar/iac/helm/templates/cronjob.yaml`, `microservices/calendar/iac/helm/templates/deployment.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `T0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `calendar` is in annual full-scope pentest and every major `T0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`, `microservices/calendar/iac/helm/Chart.yaml`, `microservices/calendar/iac/helm/templates/cronjob.yaml`, `microservices/calendar/iac/helm/templates/deployment.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `calendar` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `calendar` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `T0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `calendar` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/calendar/catalog/oya-calendar-availability-resolver-adapter-valkey.yaml`, `microservices/calendar/catalog/oya-calendar-availability-resolver-kernel.yaml`, `microservices/calendar/catalog/oya-calendar-event-store-adapter-postgres.yaml`, `microservices/calendar/catalog/oya-calendar-event-store-app.yaml`, `microservices/calendar/catalog/oya-calendar-event-store-domain.yaml`, `microservices/calendar/catalog/oya-calendar-event-store-kernel.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `T0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `calendar` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `T0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `T0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `calendar` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `T0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `calendar`; owner `axis-calendar`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `availability-resolver`, `event-store`, `ics-import-export`, `invitation-flow`, `recurrence-engine`, `room-booking`.
- Capability records cited: `microservices/calendar/capabilities/T0-suggest.yaml`, `microservices/calendar/capabilities/T1-assist.yaml`, `microservices/calendar/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar/policy artifacts cited: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`, `microservices/calendar/contracts/openapi/calendar.yaml`, `microservices/calendar/contracts/proto/calendar.proto`.
- Cedar binding: `microservices/calendar/policy/auditor-scope.cedar`, `microservices/calendar/policy/ci-scope.cedar`, `microservices/calendar/policy/data-residency.md`, `microservices/calendar/policy/event-isolation.md`, `microservices/calendar/policy/public-read.cedar`, `microservices/calendar/policy/tenant-scope.cedar`.
- State/event binding: `calendar.availability_resolver`, `calendar.event_store`, `calendar.ics_import_export`, `calendar.invitation_flow`, `calendar.recurrence_engine`, `calendar.room_booking`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/calendar/slos/agenda-render-latency.openslo.yaml`, `microservices/calendar/slos/caldav-availability.openslo.yaml`, `microservices/calendar/slos/freebusy-query-latency.openslo.yaml`, `microservices/calendar/slos/ics-import-throughput.openslo.yaml`, `microservices/calendar/slos/notification-delivery-freshness.openslo.yaml`, `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/calendar/runbooks/availability-cache-rebuild.md`, `microservices/calendar/runbooks/caldav-sync-loop.md`, `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`, `microservices/calendar/runbooks/calendar-restore.md`, `microservices/calendar/runbooks/ics-import-failure.md`, `microservices/calendar/runbooks/recurrence-storm.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `calendar`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `calendar`.
- `policy-engine` supplies the signed Cedar corpus while `calendar` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `calendar` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `calendar`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `calendar` applies the most restrictive policy and emits a degraded-mode audit event.
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
