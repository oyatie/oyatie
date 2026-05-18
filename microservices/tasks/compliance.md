---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: tasks
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0126, ADR-0140, ADR-TASKS-0006]
doc_status: published
---

# Compliance Matrix — tasks µservice

## Purpose

Enumerate compliance frameworks engaged by tasks, the controls
satisfied, and where each control is evidenced (per artifact, lane, or
runbook). Tasks introduces a high-risk-AI-touching surface (T2 auto-
assign in employment context), so the EU AI Act + EEOC + Title VII +
ADA + 직장 갑질 overlays are first-class.

## Frameworks engaged

### Globally enforced

| Framework | Scope | Mapping |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x–CC9.x | §"SOC 2 Mapping" below |
| ISO 27001:2022 | Annex A.5–A.8 | §"ISO 27001 Mapping" below |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 | §"GDPR Mapping" below |
| EU AI Act (EU) 2024/1689 | Annex III §4 + Art. 14 + Art. 50 + Art. 22 | §"EU AI Act Mapping" below |
| OWASP ASVS v4.0.3 | V1-V14 | application-security baseline |
| SLSA L3 | Supply chain | per-changeset evidence per ADR-0130 |
| NIST SSDF SP 800-218 | Secure software development | LEAN-gate program |
| WCAG 2.2 AA | Accessibility | per `microservices/tasks/legal/accessibility.md` |
| CIS Kubernetes Benchmark | Substrate | `cloud-k8s` inheritance |
| ISO 30414 | HR analytics | guidance for time-tracking + auto-assign |

### Pack-overlays

| Pack | Frameworks |
|---|---|
| pack-kr | KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2/33 + ISMS-P §2.1-2.12 + 근로기준법 Arts. 23/41 + 직장 갑질 protections |
| pack-us-healthcare | HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502 + state-level (CCPA / CMIA / NY SHIELD) + FDA 21 CFR Part 11 + ADA + EEOC UGESP 1978 + Title VII |
| pack-eu | GDPR + EDPB Guidelines 4/2019 + 9/2022 + NIS2 + eIDAS 910/2014 + EU AI Act Annex III §4 |
| pack-us | CCPA / CPRA + EEOC UGESP 1978 + Title VII + ADA + state-level workplace-surveillance laws (NY, IL, CT, CO, CA) |
| pack-jp | APPI Arts. 17/18/20/21/23/24/26-2/27 + Japanese Labour Standards Act (employment-context auto-assign) |
| pack-sg | PDPA 2012 Parts III/IV/VI + Employment Act |
| pack-au | Privacy Act 1988 APP 1-13 + Fair Work Act 2009 (employment-context surveillance) |
| pack-in | DPDPA 2023 §6-11 + Industrial Disputes Act 1947 |
| pack-br | LGPD Arts. 6/7/11/14/18/33/38/46/48 + CLT (employment context) |
| pack-ae | UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15/22/23 + UAE Labour Law |
| pack-ksa | KSA PDPL Royal Decree M/19/2021 Arts. 4-9 + KSA Labour Law |

## SOC 2 Mapping

| TSC | Control | Tasks evidence |
|---|---|---|
| CC1.1 | Demonstrates commitment to integrity | council sign-off on threat-model + DPIA |
| CC2.1 | Communicates information | This compliance.md + PRD + runbooks |
| CC3.1 | Specifies risk objectives | threat-model.md + dpia.md |
| CC4.1 | Demonstrates evaluation | audit-chain emission per task + LEAN check coverage |
| CC4.2 | Selects monitoring activities | observability dashboards + per-changeset evidence |
| CC5.1 | Selects + develops control activities | LEAN CI lanes + Cedar policies |
| CC6.1 | Logical access — restricts | per-tenant RLS + tenant-DEK + Cedar |
| CC6.2 | Authenticates | OIDC + MFA + per-tenant API keys |
| CC6.3 | Authorises | Cedar policies (`tenant-scope.cedar`, etc.) |
| CC6.6 | Restricts physical access | inherited from cloud-k8s + cloud-secrets |
| CC6.7 | Restricts info flow | type-narrowed projections + LEAN checks |
| CC6.8 | Prevents unauthorized software | branch-protection + signed commits + signed artifacts + SLSA L3 |
| CC7.1 | Detects security events | observability alerts + audit-chain |
| CC7.2 | Monitors system components | SLO + burn-rate dashboards |
| CC7.4 | Responds to incidents | incident-response.md + runbooks |
| CC7.5 | Recovers from incidents | DR drills + backup retention |
| CC8.1 | Manages changes | ADR-0110 ChangeSet + ADR-0130 SLO-gated promotion |
| CC9.1 | Identifies risks | residual-risk acceptance in threat-model.md |

## ISO 27001 Mapping

| Annex A Control | Tasks evidence |
|---|---|
| A.5.7 (threat intelligence) | threat-model.md re-review triggers |
| A.5.10 (acceptable use of info) | policy/task-isolation.md |
| A.5.14 (information transfer) | policy/data-residency.md |
| A.5.15 (access control) | Cedar policies + RLS |
| A.5.17 (authentication info) | OpenBao + per-tenant DEK rotation |
| A.5.23 (cloud service usage) | ADR-0117 |
| A.5.26 (response to security incidents) | incident-response.md |
| A.5.27 (lessons from incidents) | post-incident review process |
| A.5.28 (collection of evidence) | audit-chain seal |
| A.5.30 (ICT continuity) | multi-region.md |
| A.5.31 (legal + statutory) | This compliance.md per-pack overlays |
| A.5.33 (records protection) | retention + legal-hold |
| A.5.34 (privacy + PII) | dpia.md + policy/* |
| A.8.2 (privileged access rights) | OpenBao JIT + 2-person rule |
| A.8.3 (info access restriction) | RLS + Cedar |
| A.8.5 (secure authentication) | OIDC + MFA |
| A.8.7 (protection against malware) | importer subprocess sandbox; inherited from cloud-k8s |
| A.8.11 (data masking) | redaction in export per Cedar projection |
| A.8.12 (data leakage prevention) | LEAN checks + DLP scan |
| A.8.15 (logging) | observability + audit-chain |
| A.8.16 (monitoring activities) | dashboards + alerts |
| A.8.20 (network security) | mesh mTLS + NetworkPolicy |
| A.8.21 (security of network services) | per-tenant API key + rate limits |
| A.8.23 (web filtering) | WAF at ingress |
| A.8.25 (secure development lifecycle) | LEAN gates + ADR-0130 SLO-gated promotion + SLSA L3 |
| A.8.26 (application security requirements) | per-microservice security artifacts |
| A.8.27 (secure system architecture) | ADR-0056 + ADR-0105 |
| A.8.28 (secure coding) | LEAN check `oya-check-importer-sandbox-config` + `oya-check-custom-field-type-strict` + cargo fuzz |
| A.8.32 (change management) | ADR-0110 + branch-protection |
| A.8.33 (test information) | synthetic test tenants per `ci-scope.cedar` |
| A.8.34 (audit + protection of audit systems) | audit-chain immutability + 2-person rule on admin |

## GDPR Mapping

| Article | Tasks evidence |
|---|---|
| Art. 5(1)(a) lawfulness | per-purpose lawful basis in dpia.md |
| Art. 5(1)(b) purpose limitation | dpia.md §2.4 |
| Art. 5(1)(c) data minimisation | type-narrowed projections; webhook field-projection per subscription |
| Art. 5(1)(d) accuracy | tenant-edit UX + audit history |
| Art. 5(1)(e) storage limitation | retention per pack |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK + Ed25519 audit-chain |
| Art. 6(1) lawful basis | dpia.md §2.4 |
| Art. 9 special-category | pack-us-healthcare overlay (PHI in clinical tasks) |
| Art. 13/14 transparency | tenant DPA template + EU AI Act Art. 50 user labelling for T1/T2 |
| Art. 17 right-to-erasure | DSR cascade + hold-vs-erasure policy |
| Art. 22 automated decision (employment-context auto-assign) | Cedar refusal until ADR-TASKS-0006 conformity ADR; Art. 14 reversibility-window override |
| Art. 25 by design + default | type-system separation + Cedar policy + dependency-graph cycle prevention |
| Art. 28 processor agreement | DPA template |
| Art. 30 records of processing | RoPA template |
| Art. 32 security of processing | every STRIDE / LINDDUN mitigation |
| Art. 33 breach notification | incident-response.md 72h chain |
| Art. 35 DPIA | dpia.md |
| Arts. 44-50 transfers | SCC + transfer register + multi-region.md |

## EU AI Act Mapping

| Article / Annex | Tasks evidence |
|---|---|
| **Annex III §4 (employment-context, including recruitment, performance evaluation, allocation of tasks)** | **T2 auto-assign in employment-context REFUSED at Cedar layer pending ADR-TASKS-0006 conformity-assessment ADR**. T0/T1 capabilities (suggest only, classify only) outside Annex III scope. |
| Art. 9 (risk-management system) | ADR-TASKS-0006 + threat-model.md per-capability promotion lifecycle |
| Art. 10 (data governance) | data-class enforcement + bias audit per `slos/auto-assign-fairness-correctness.openslo.yaml` |
| Art. 13 (transparency to providers) | This compliance.md + DPIA |
| Art. 14 (human oversight) | T1/T2 reversibility window 30s per `capabilities/T1-assist.yaml` + `T2-auto.yaml` |
| Art. 15 (accuracy + robustness + cybersecurity) | per-decision Ed25519 audit chain + foundry-runtime quality gates |
| Art. 50 (transparency obligations re. AI interactions) | UI labelling per `capabilities/T0-suggest.yaml` + `T1-assist.yaml` + `T2-auto.yaml` `user_label` field |
| Annex IV (technical documentation) | This compliance + DPIA + ADR-TASKS-0006 |

## Pack-overlay detail: pack-kr (KR PIPA + 근로기준법 + 직장 갑질 + ISMS-P + 전자문서법)

| Citation | Tasks implementation |
|---|---|
| KR PIPA Art. 15 (consent) | tenant onboarding consent flow |
| KR PIPA Art. 17 (cross-border) | default-residency + SCC clause |
| KR PIPA Art. 23 (sensitive) | sensitive-flag per-task; access restrictions via Cedar |
| KR PIPA Art. 28 (storage period) | retention bounded per asset table |
| KR PIPA Art. 29 (technical safeguards) | 12-safeguard mapping per threat-model.md |
| KR PIPA Art. 33 (DPIA / 영향평가) | dpia.md |
| 근로기준법 Art. 41 (employment records retention 3y minimum) | retention floor 1095d for employment-context tasks |
| 근로기준법 Art. 23 (anti-discrimination) | auto-assign fairness audit; refused without |
| 직장 갑질 protections | T2 auto-assign of high-workload tasks REFUSED at Cedar layer for KR employment context |
| ISMS-P §2.1-2.12 | per ISMS-P annual recert; Meilisearch within scope |
| 전자문서법 Art. 5 | Ed25519 audit-chain |

## Pack-overlay detail: pack-us-healthcare (HIPAA + ADA + EEOC + state)

| Citation | Tasks implementation |
|---|---|
| HIPAA §164.308(a)(1)(ii)(A) risk analysis | dpia.md + threat-model.md |
| HIPAA §164.308(a)(3) workforce security | OpenBao JIT + 2-person rule |
| HIPAA §164.308(a)(4) info access | Cedar + RLS |
| HIPAA §164.310(a) facility | inherited from cloud-k8s |
| HIPAA §164.312(a) access control | RLS + Cedar |
| HIPAA §164.312(b) audit controls | Ed25519 + retention ≥6y |
| HIPAA §164.312(c) integrity | audit-chain Merkle |
| HIPAA §164.312(d) person auth | OIDC + MFA |
| HIPAA §164.312(e) transmission | mesh mTLS |
| HIPAA §164.314(a) BAA | legal/baa-template.md |
| HIPAA §164.316 documentation | retain artifacts ≥6y |
| HIPAA §164.502(a) Permitted Uses (TPO) | tenant DPA |
| HIPAA §164.502(b) Minimum Necessary | data-class enforcement |
| HIPAA §164.504(e) BAA terms | BAA template |
| ADA 42 USC §12101 (accommodation tasks) | accessibility per WCAG 2.2 AA |
| EEOC UGESP 1978 (29 CFR §1607) | T2 auto-assign in employment-context REFUSED at Cedar layer for pack-us until fairness-audit complete |
| Title VII Civil Rights Act 1964 | per EEOC UGESP |
| FDA 21 CFR Part 11 | when clinical-task touches research subjects, audit-chain seal satisfies e-signature |

State-level:
- CCPA §1798.100 et seq.: DSR cascade satisfies.
- CMIA §56 et seq.: medical-info disclosure restrictions; pack-us-healthcare enforces.
- NY SHIELD Act, IL BIPA + AI Video Interview Act, CT SB-1103, CO AI Act, CA AB-331 (proposed): per-state workplace-AI assignment regulations; tasks µservice's Cedar refusal of T2 auto-assign covers pack-us until per-state activation ADR.

## Pack-overlay detail: pack-eu (GDPR + EDPB + NIS2 + eIDAS + EU AI Act)

- **GDPR Art. 25**: privacy-by-design baked into Rust type system + Cedar refusal of T2 auto-assign.
- **GDPR Art. 35**: DPIA in `dpia.md` satisfies; EU AI Act Annex III §4 triggers automatic-DPIA per Art. 35(3)(a).
- **GDPR Art. 28 (processor)**: tenant DPA template + sub-processor list.
- **GDPR Art. 32**: every STRIDE mitigation contributes.
- **GDPR Arts. 44–50 (transfers)**: pack-eu Postgres cluster EU-resident; cross-region replication forbidden by default.
- **GDPR Art. 22 (automated decisions)**: T2 auto-assign in employment-context refused at Cedar layer until conformity ADR ships + Art. 14 reversibility window when permitted.
- **EU AI Act Art. 9 (risk management)**: ADR-TASKS-0006 + threat-model.md per-capability promotion lifecycle.
- **EU AI Act Art. 14 (human oversight)**: T1/T2 reversibility-window 30s.
- **EU AI Act Art. 50 (transparency)**: user-labelling per `capabilities/*.yaml` `user_label` field.
- **EU AI Act Annex IV (technical documentation)**: this compliance + DPIA + ADR-TASKS-0006.
- **NIS2**: incident-response timelines (24h+72h+1mo) when oyatie crosses thresholds.
- **eIDAS 910/2014**: audit-chain Ed25519 seals are AdES.

## Pack-overlay detail: pack-us (CCPA + CPRA + EEOC + Title VII + ADA + state workplace AI)

| Citation | Tasks implementation |
|---|---|
| CCPA §1798.100 (right to know) | per-user export per PRD FR-10 |
| CCPA §1798.105 (right to delete) | task-store-usecase deletion orchestrator |
| CCPA §1798.120 (sale opt-out) | no sale; documented in `legal/sub-processors.md` |
| EEOC UGESP 1978 (29 CFR §1607) | T2 auto-assign in employment-context REFUSED until fairness-audit complete per `slos/auto-assign-fairness-correctness.openslo.yaml` |
| Title VII Civil Rights Act 1964 | per EEOC UGESP |
| ADA 42 USC §12101 | accessibility per WCAG 2.2 AA |
| NY Local Law 144 (AEDT) | bias-audit for any automated-employment-decision-tool (AEDT); T2 auto-assign refused for pack-us-NY until AEDT audit complete |
| IL AI Video Interview Act | N/A (tasks doesn't do video) |
| CO AI Act (HB23-1041) | T2 auto-assign refused until disclosure + opt-out wired |
| SOC 2 | annual SOC 2 Type 2 |

## Pack-overlay detail: pack-jp (APPI + Labour Standards Act)

| Citation | Tasks implementation |
|---|---|
| APPI Art. 17 (purpose) | tenant DPA |
| APPI Art. 21 (cross-border) | pack-pinning |
| APPI Art. 23 (joint use) | tenant DPA |
| APPI Art. 24 (third-party provision) | sub-processor list |
| APPI Art. 26-2 (cross-border consent) | tenant DPA |
| Japanese Labour Standards Act (employment-record retention) | retention floor for employment-context tasks |

## Pack-overlay detail: pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/tasks-compliance-overlay.md`.

Highlights:
- **PDPA 2012**: Part III Protection + Part IV Retention + Part VI Transfer + Employment Act.
- **APP 8 + APP 11 + APP 12** (Privacy Act 1988): cross-border + security + access; Fair Work Act 2009 for assignment-context.
- **DPDPA 2023**: §6-11 consent/notice/security; Industrial Disputes Act 1947 for employment-context.
- **LGPD**: Arts. 33-36 cross-border + CLT (Consolidação das Leis do Trabalho) for employment-context.
- **UAE PDPL + KSA PDPL**: cross-border + impact assessment + UAE / KSA Labour Law for employment-context.

## Audit cadence

| Audit | Cadence | Owner |
|---|---|---|
| SOC 2 Type 2 | annually | external SOC 2 firm |
| ISO 27001:2022 | every 3 years (initial) + annual surveillance | external ISO firm |
| GDPR / EDPB DPA examination | on-tenant request + breach trigger | council-privacy |
| EU AI Act conformity assessment (T2 auto-assign) | pre-promotion + annually | external notified body (when conformity scope activated) |
| HIPAA OCR audit | on-trigger | external HIPAA firm |
| PIPC examination | on-trigger | council-privacy |
| EEOC bias-audit (T2 auto-assign in pack-us employment) | pre-promotion + annually | external bias-audit firm |
| NY Local Law 144 AEDT bias-audit | annually | external NYC-DCWP-registered firm |
| Pen-test (boundary tests per threat-model.md) | annually | external pen-test firm |
| LEAN-gate evidence review | per-PR | reviewer-agent |
| WCAG 2.2 AA accessibility audit | annually + per-feature-launch | accessibility consultancy |

## References

- ADR-0028 (Bominal), ADR-0117, ADR-0126, ADR-0140, ADR-TASKS-0006.
- `threat-model.md`, `dpia.md`, `policy/*`, `multi-region.md`, `incident-response.md`, `legal/*`.
- SOC 2 TSC 2017 + 2022 PoF; ISO 27001:2022 Annex A.
- GDPR (Regulation 2016/679); EDPB Guidelines 4/2019 + 9/2022.
- EU AI Act Regulation (EU) 2024/1689 — Annex III §4 + Art. 14 + Art. 50 + Art. 22.
- KR PIPA + 근로기준법 + 직장 갑질 protections + ISMS-P + 전자문서법; PIPC Notice 2020-7.
- HIPAA 45 CFR §164; FDA 21 CFR Part 11; ADA 42 USC §12101.
- EEOC UGESP 1978 (29 CFR §1607); Title VII Civil Rights Act 1964; NY Local Law 144; IL AI Video Interview Act; CO AI Act HB23-1041; CA AB-331 (proposed).
- APPI; PDPA; APP; DPDPA; LGPD; UAE PDPL; KSA PDPL.
- ISO 30414 (HR analytics).
- WCAG 2.2 AA.
- OWASP ASVS v4.0.3; SLSA L3; NIST SSDF SP 800-218.
- `microservices/calendar/compliance.md` — sibling reference template.
