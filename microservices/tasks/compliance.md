---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: tasks
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140 (retired per ADR-0145), ADR-TASKS-0006]
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
| SLSA L3 | Supply chain | per-changeset evidence per ADR-0139 |
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
| CC8.1 | Manages changes | ADR-0110 ChangeSet + ADR-0139 SLO-gated promotion |
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
| A.8.25 (secure development lifecycle) | LEAN gates + ADR-0139 SLO-gated promotion + SLSA L3 |
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

- ADR-0028 (Bominal), ADR-0117, ADR-0135, ADR-0140, ADR-TASKS-0006.
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

---



## §day-one-cert-readiness
This anchor is closed for `tasks` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `tasks` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +13 more.
- Example: `T0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `T0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against ADR-0292 §D-1: minor-user refusal, teen handling class and age-verification handling.

### Service-specific answer
- Minor exposure for `tasks` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `T0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen handling activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `tasks` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`, `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `T0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.tasks.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `tasks` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `T0-suggest` touches those data classes.
- Signal sources: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +13 more.
- Example event class: `oya.tasks.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `tasks` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.tasks.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `T0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `T0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `tasks` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`, `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`; +4 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `tasks.dependency_graph` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for ADR-0248 Tier 0/1 cell-criticality paths, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `tasks` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +13 more.
- Example: `T0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.tasks` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/tasks/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.
- Example: `T0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for ADR-0248 Tier 0/1 cell-criticality paths and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `tasks` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`, `microservices/tasks/iac/helm/tasks/Chart.yaml`, `microservices/tasks/iac/helm/tasks/templates/deployment.yaml`, `microservices/tasks/iac/helm/tasks/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `T0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `tasks` is in annual full-scope pentest and every major `T0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`, `microservices/tasks/iac/helm/tasks/Chart.yaml`, `microservices/tasks/iac/helm/tasks/templates/deployment.yaml`, `microservices/tasks/iac/helm/tasks/templates/hpa.yaml`; +16 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `tasks` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `tasks` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether ADR-0248 Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `T0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `tasks` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/tasks/catalog/oya-tasks-dependency-graph-kernel.yaml`, `microservices/tasks/catalog/oya-tasks-importers-adapter-jira.yaml`, `microservices/tasks/catalog/oya-tasks-importers-kernel.yaml`, `microservices/tasks/catalog/oya-tasks-project-list-adapter-postgres.yaml`, `microservices/tasks/catalog/oya-tasks-project-list-kernel.yaml`, `microservices/tasks/catalog/oya-tasks-project-list-rest.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `T0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `tasks` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `T0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `T0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `tasks` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `T0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `tasks`; owner `axis-tasks`; service_class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dependency-graph`, `importers`, `project-list`, `recurrence`, `search-index`, `task-store`; +1 more.
- Capability records cited: `microservices/tasks/capabilities/T0-suggest.yaml`, `microservices/tasks/capabilities/T1-assist.yaml`, `microservices/tasks/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar/policy artifacts cited: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- SLO and dashboard evidence: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/tasks/contracts/asyncapi/tasks-events.yaml`, `microservices/tasks/contracts/openapi/tasks.yaml`, `microservices/tasks/contracts/proto/tasks.proto`.
- Cedar binding: `microservices/tasks/policy/auditor-scope.cedar`, `microservices/tasks/policy/ci-scope.cedar`, `microservices/tasks/policy/data-residency.md`, `microservices/tasks/policy/dual-context-isolation.cedar`, `microservices/tasks/policy/public-read.cedar`, `microservices/tasks/policy/task-isolation.md`; +1 more.
- State/event binding: `tasks.dependency_graph`, `tasks.importers`, `tasks.project_list`, `tasks.recurrence`, `tasks.search_index`, `tasks.task_store`; +1 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/tasks/slos/auto-assign-fairness-correctness.openslo.yaml`, `microservices/tasks/slos/bulk-update-latency.openslo.yaml`, `microservices/tasks/slos/dependency-cycle-detection-correctness.openslo.yaml`, `microservices/tasks/slos/recurring-materialise-latency.openslo.yaml`, `microservices/tasks/slos/search-latency.openslo.yaml`, `microservices/tasks/slos/task-create-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/tasks/runbooks/ai-assign-classifier-rollback.md`, `microservices/tasks/runbooks/bulk-edit-throttle.md`, `microservices/tasks/runbooks/custom-field-schema-migration.md`, `microservices/tasks/runbooks/dependency-cycle-corruption.md`, `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`, `microservices/tasks/runbooks/search-index-rebuild.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `tasks`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `tasks`.
- `policy-engine` supplies the signed Cedar corpus while `tasks` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `tasks` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `tasks`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `tasks` applies the most restrictive policy and emits a degraded-mode audit event.
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
